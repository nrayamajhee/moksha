use super::{prelude::*, Object, SceneObject, Storage};
use crate::{Color, Id, RcRcell};

use moksha_derive::Object;
use strum_macros::{Display, EnumIter, EnumString};

#[derive(Object, Clone)]
pub struct Light {
    obj_id: Id,
    light_id: Id,
    storage: RcRcell<Storage>,
}

impl Light {
    pub fn new(storage: RcRcell<Storage>, light_id: Id, obj_id: Id) -> Self {
        let obj_id = storage.borrow().lights()[light_id].obj_id;
        Self {
            obj_id,
            light_id,
            storage
        }
    }
}

impl Light {
    pub fn set_light_info(&self, info: LightInfo) {
        let mut storage = self.storage.borrow_mut();
        let light_info = storage.mut_light_info(self.light_id);
        *light_info = info
    }
    pub fn light_info(&self) -> LightInfo {
        let storage = self.storage.borrow();
        storage.light(self.light_id)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct LightInfo {
    pub light_type: LightType,
    pub state: LightState,
    pub intensity: f32,
    pub color: Color,
    pub obj_id: Id,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Display, EnumIter, EnumString)]
pub enum LightType {
    Ambient,
    Point,
    Directional,
    Spot,
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LightState {
    On,
    Off,
}

impl Into<bool> for LightState {
    fn into(self) -> bool {
        match self {
            LightState::On => true,
            LightState::Off => false,
        }
    }
}
