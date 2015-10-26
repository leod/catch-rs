use std::iter::Iterator;

use rustc_serialize::{Encoder, Decoder, Encodable, Decodable};
use ecs::{ComponentManager, BuildData, EntityData};

use components::*;

pub type ComponentsBitSet = u16;

macro_rules! net_components {
    {
        struct $Name:ident {
            $($field_name:ident, $field_name_mut:ident : $field_ty:ident, $field_trait:ident),+,
        }
        enum $EnumName:ident;
        const $TypesName:ident;
    } => {
        #[derive(Default, Clone)]
        pub struct $Name {
            $(
                pub $field_name : Option<$field_ty>,
            )+
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, RustcEncodable, RustcDecodable)]
        pub enum $EnumName {
            $(
                $field_ty,         
            )+
        }

        pub const $TypesName: &'static [$EnumName] = &[
            $(
                $EnumName::$field_ty,
            )+
        ];

        impl $Name {
            #[allow(unused_assignments)] 
            pub fn encode<S: Encoder>
                         (&self, s: &mut S)
                         -> Result<(), S::Error> {
                let mut bit_set: ComponentsBitSet = 0;
                let mut i = 0;
                $(
                    if self.$field_name.is_some() {
                        bit_set |= 1 << i;
                    }
                    i += 1;
                )+
                try!(bit_set.encode(s));

                $(
                    if let Some(f) = self.$field_name.as_ref() {
                        try!(f.encode(s));
                    }
                )+

                Ok(())
            }

            #[allow(unused_assignments)]
            pub fn delta_encode<S: Encoder>
                               (&self,
                                neq_components: ComponentsBitSet,
                                s: &mut S)
                                -> Result<(), S::Error> {
                try!(neq_components.encode(s));

                let mut i = 0;
                $(
                    if (neq_components >> i) & 1 == 1 {
                        try!(self.$field_name.as_ref().unwrap().encode(s));
                    }
                    i += 1;
                )+

                Ok(())
            }

            #[allow(unused_assignments)] 
            pub fn decode<D: Decoder>
                         (d: &mut D)
                         -> Result<$Name, D::Error> {
                let bit_set = try!(ComponentsBitSet::decode(d));

                let mut e = $Name::default();
                let mut i = 0;
                $(
                    if (bit_set >> i) & 1 == 1 {
                        let c = try!($field_ty::decode(d));
                        e.$field_name = Some(c); 
                    }
                    i += 1;
                )+

                Ok(e)
            }

            #[allow(unused_assignments)] 
            pub fn neq_components(&self, other: &NetComponents) -> ComponentsBitSet {
                let mut bit_set: ComponentsBitSet = 0;
                let mut i = 0;
                $(
                    match (&self.$field_name, &other.$field_name) {
                        (&Some(ref a), &Some(ref b)) => {
                            if a != b {
                                bit_set |= 1 << i;
                            }
                        }
                        (&None, &None) => (),
                        (_, _) => panic!("this shouldn't happen (I hope)"),
                    }
                    i += 1;
                )+
                bit_set
            }

            pub fn add_component<C: ComponentManager>
                                (component: $EnumName,
                                 entity: BuildData<C>,
                                 data: &mut C) 
                where C: $($field_trait +)+ {
                match component {
                    $(
                        $EnumName::$field_ty => {
                            data.$field_name_mut().add(&entity, $field_ty::default());
                        }
                    )+
                }
            }

            pub fn from_entity<C: ComponentManager,
                               I: Iterator<Item=$EnumName>>
                              (components: I,
                               entity: EntityData<C>,
                               data: &C) -> $Name
                where C: $($field_trait +)+ {
                let mut e = $Name::default();
                for c in components {
                    match c {
                        $(
                            $EnumName::$field_ty => 
                                e.$field_name = Some(data.$field_name()[entity].clone()),
                        )+
                    }
                }
                e
            }

            pub fn load_to_entity<C: ComponentManager,
                                  I: Iterator<Item=$EnumName>>
                                 (&self,
                                  components: I,
                                  entity: EntityData<C>,
                                  data: &mut C)
                where C: $($field_trait +)+ {
                for c in components {
                    match c {
                        $(
                            $EnumName::$field_ty => 
                                data.$field_name_mut()[entity] =
                                    self.$field_name.as_ref().unwrap().clone(),
                        )+
                    }
                }
            }

            pub fn load_delta(&mut self, new_state: &NetComponents) {
                $(
                    if let Some(x) = new_state.$field_name.as_ref() {
                        self.$field_name = Some(x.clone());
                    }
                )+
            }
        }
    };
}

// Components whose state can be synchronized over the net
net_components! {
    struct NetComponents {
        position, position_mut: Position, HasPosition,
        orientation, orientation_mut: Orientation, HasOrientation,
        linear_velocity, linear_velocity_mut: LinearVelocity, HasLinearVelocity,
        shape, shape_mut: Shape, HasShape,
        player_state, player_state_mut: PlayerState, HasPlayerState,
        full_player_state, full_player_state_mut: FullPlayerState, HasFullPlayerState,
        wall_position, wall_position_mut: WallPosition, HasWallPosition,
    }
    enum ComponentType;
    const COMPONENT_TYPES;
}
