/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(u8)]
pub enum CatSoundVariant {
    #[default]
    Classic = 0u8,
    Royal = 1u8,
}
impl CatSoundVariant {
    pub const ALL: &'static [Self] = &[Self::Classic, Self::Royal];
    #[doc = "Returns the sound variant from a resource name (bare or namespaced)."]
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "minecraft:classic" | "classic" => Some(Self::Classic),
            "minecraft:royal" | "royal" => Some(Self::Royal),
            _ => None,
        }
    }
    #[doc = "Returns the numeric ID of the sound variant in the synced registry."]
    #[must_use]
    pub const fn id(&self) -> u8 {
        *self as u8
    }
    #[must_use]
    pub const fn from_id(id: u8) -> Option<Self> {
        match id {
            0u8 => Some(Self::Classic),
            1u8 => Some(Self::Royal),
            _ => None,
        }
    }
    #[doc = "Returns the bare string name of the sound variant."]
    #[must_use]
    pub const fn to_name(&self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Royal => "royal",
        }
    }
    #[must_use]
    pub const fn ambient_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityCatAmbient
                } else {
                    crate::sound::Sound::EntityCatAmbient
                }
            }
            Self::Royal => {
                if is_baby {
                    crate::sound::Sound::EntityCatAmbient
                } else {
                    crate::sound::Sound::EntityCatAmbient
                }
            }
        }
    }
    #[must_use]
    pub const fn death_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityCatDeath
                } else {
                    crate::sound::Sound::EntityCatDeath
                }
            }
            Self::Royal => {
                if is_baby {
                    crate::sound::Sound::EntityCatDeath
                } else {
                    crate::sound::Sound::EntityCatDeath
                }
            }
        }
    }
    #[must_use]
    pub const fn hurt_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityCatHurt
                } else {
                    crate::sound::Sound::EntityCatHurt
                }
            }
            Self::Royal => {
                if is_baby {
                    crate::sound::Sound::EntityCatHurt
                } else {
                    crate::sound::Sound::EntityCatHurt
                }
            }
        }
    }
    #[must_use]
    pub const fn eat_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityCatEat
                } else {
                    crate::sound::Sound::EntityCatEat
                }
            }
            Self::Royal => {
                if is_baby {
                    crate::sound::Sound::EntityCatEat
                } else {
                    crate::sound::Sound::EntityCatEat
                }
            }
        }
    }
    #[must_use]
    pub const fn hiss_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityCatHiss
                } else {
                    crate::sound::Sound::EntityCatHiss
                }
            }
            Self::Royal => {
                if is_baby {
                    crate::sound::Sound::EntityCatHiss
                } else {
                    crate::sound::Sound::EntityCatHiss
                }
            }
        }
    }
    #[must_use]
    pub const fn purr_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityCatPurr
                } else {
                    crate::sound::Sound::EntityCatPurr
                }
            }
            Self::Royal => {
                if is_baby {
                    crate::sound::Sound::EntityCatPurr
                } else {
                    crate::sound::Sound::EntityCatPurr
                }
            }
        }
    }
    #[must_use]
    pub const fn purreow_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityCatPurreow
                } else {
                    crate::sound::Sound::EntityCatPurreow
                }
            }
            Self::Royal => {
                if is_baby {
                    crate::sound::Sound::EntityCatPurreow
                } else {
                    crate::sound::Sound::EntityCatPurreow
                }
            }
        }
    }
    #[must_use]
    pub const fn beg_for_food_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityCatBegForFood
                } else {
                    crate::sound::Sound::EntityCatBegForFood
                }
            }
            Self::Royal => {
                if is_baby {
                    crate::sound::Sound::EntityCatBegForFood
                } else {
                    crate::sound::Sound::EntityCatBegForFood
                }
            }
        }
    }
    #[must_use]
    pub const fn stray_ambient_sound(&self) -> crate::sound::Sound {
        match self {
            Self::Classic => crate::sound::Sound::EntityCatStrayAmbient,
            Self::Royal => crate::sound::Sound::EntityCatStrayAmbient,
        }
    }
    #[must_use]
    pub const fn all() -> &'static [Self] {
        Self::ALL
    }
}
