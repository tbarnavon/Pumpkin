/* This file is generated. Do not edit manually. */
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[repr(u8)]
pub enum WolfSoundVariant {
    Angry = 0u8,
    Big = 1u8,
    #[default]
    Classic = 2u8,
    Cute = 3u8,
    Grumpy = 4u8,
    Puglin = 5u8,
    Sad = 6u8,
}
impl WolfSoundVariant {
    pub const ALL: &'static [Self] = &[
        Self::Angry,
        Self::Big,
        Self::Classic,
        Self::Cute,
        Self::Grumpy,
        Self::Puglin,
        Self::Sad,
    ];
    #[doc = "Returns the sound variant from a resource name (bare or namespaced)."]
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "minecraft:angry" | "angry" => Some(Self::Angry),
            "minecraft:big" | "big" => Some(Self::Big),
            "minecraft:classic" | "classic" => Some(Self::Classic),
            "minecraft:cute" | "cute" => Some(Self::Cute),
            "minecraft:grumpy" | "grumpy" => Some(Self::Grumpy),
            "minecraft:puglin" | "puglin" => Some(Self::Puglin),
            "minecraft:sad" | "sad" => Some(Self::Sad),
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
            0u8 => Some(Self::Angry),
            1u8 => Some(Self::Big),
            2u8 => Some(Self::Classic),
            3u8 => Some(Self::Cute),
            4u8 => Some(Self::Grumpy),
            5u8 => Some(Self::Puglin),
            6u8 => Some(Self::Sad),
            _ => None,
        }
    }
    #[doc = "Returns the bare string name of the sound variant."]
    #[must_use]
    pub const fn to_name(&self) -> &'static str {
        match self {
            Self::Angry => "angry",
            Self::Big => "big",
            Self::Classic => "classic",
            Self::Cute => "cute",
            Self::Grumpy => "grumpy",
            Self::Puglin => "puglin",
            Self::Sad => "sad",
        }
    }
    #[must_use]
    pub const fn ambient_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Angry => {
                if is_baby {
                    crate::sound::Sound::EntityWolfAmbient
                } else {
                    crate::sound::Sound::EntityWolfAmbient
                }
            }
            Self::Big => {
                if is_baby {
                    crate::sound::Sound::EntityWolfAmbient
                } else {
                    crate::sound::Sound::EntityWolfAmbient
                }
            }
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityWolfAmbient
                } else {
                    crate::sound::Sound::EntityWolfAmbient
                }
            }
            Self::Cute => {
                if is_baby {
                    crate::sound::Sound::EntityWolfAmbient
                } else {
                    crate::sound::Sound::EntityWolfAmbient
                }
            }
            Self::Grumpy => {
                if is_baby {
                    crate::sound::Sound::EntityWolfAmbient
                } else {
                    crate::sound::Sound::EntityWolfAmbient
                }
            }
            Self::Puglin => {
                if is_baby {
                    crate::sound::Sound::EntityWolfAmbient
                } else {
                    crate::sound::Sound::EntityWolfAmbient
                }
            }
            Self::Sad => {
                if is_baby {
                    crate::sound::Sound::EntityWolfAmbient
                } else {
                    crate::sound::Sound::EntityWolfAmbient
                }
            }
        }
    }
    #[must_use]
    pub const fn death_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Angry => {
                if is_baby {
                    crate::sound::Sound::EntityWolfDeath
                } else {
                    crate::sound::Sound::EntityWolfDeath
                }
            }
            Self::Big => {
                if is_baby {
                    crate::sound::Sound::EntityWolfDeath
                } else {
                    crate::sound::Sound::EntityWolfDeath
                }
            }
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityWolfDeath
                } else {
                    crate::sound::Sound::EntityWolfDeath
                }
            }
            Self::Cute => {
                if is_baby {
                    crate::sound::Sound::EntityWolfDeath
                } else {
                    crate::sound::Sound::EntityWolfDeath
                }
            }
            Self::Grumpy => {
                if is_baby {
                    crate::sound::Sound::EntityWolfDeath
                } else {
                    crate::sound::Sound::EntityWolfDeath
                }
            }
            Self::Puglin => {
                if is_baby {
                    crate::sound::Sound::EntityWolfDeath
                } else {
                    crate::sound::Sound::EntityWolfDeath
                }
            }
            Self::Sad => {
                if is_baby {
                    crate::sound::Sound::EntityWolfDeath
                } else {
                    crate::sound::Sound::EntityWolfDeath
                }
            }
        }
    }
    #[must_use]
    pub const fn hurt_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Angry => {
                if is_baby {
                    crate::sound::Sound::EntityWolfHurt
                } else {
                    crate::sound::Sound::EntityWolfHurt
                }
            }
            Self::Big => {
                if is_baby {
                    crate::sound::Sound::EntityWolfHurt
                } else {
                    crate::sound::Sound::EntityWolfHurt
                }
            }
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityWolfHurt
                } else {
                    crate::sound::Sound::EntityWolfHurt
                }
            }
            Self::Cute => {
                if is_baby {
                    crate::sound::Sound::EntityWolfHurt
                } else {
                    crate::sound::Sound::EntityWolfHurt
                }
            }
            Self::Grumpy => {
                if is_baby {
                    crate::sound::Sound::EntityWolfHurt
                } else {
                    crate::sound::Sound::EntityWolfHurt
                }
            }
            Self::Puglin => {
                if is_baby {
                    crate::sound::Sound::EntityWolfHurt
                } else {
                    crate::sound::Sound::EntityWolfHurt
                }
            }
            Self::Sad => {
                if is_baby {
                    crate::sound::Sound::EntityWolfHurt
                } else {
                    crate::sound::Sound::EntityWolfHurt
                }
            }
        }
    }
    #[must_use]
    pub const fn step_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Angry => {
                if is_baby {
                    crate::sound::Sound::EntityWolfStep
                } else {
                    crate::sound::Sound::EntityWolfStep
                }
            }
            Self::Big => {
                if is_baby {
                    crate::sound::Sound::EntityWolfStep
                } else {
                    crate::sound::Sound::EntityWolfStep
                }
            }
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityWolfStep
                } else {
                    crate::sound::Sound::EntityWolfStep
                }
            }
            Self::Cute => {
                if is_baby {
                    crate::sound::Sound::EntityWolfStep
                } else {
                    crate::sound::Sound::EntityWolfStep
                }
            }
            Self::Grumpy => {
                if is_baby {
                    crate::sound::Sound::EntityWolfStep
                } else {
                    crate::sound::Sound::EntityWolfStep
                }
            }
            Self::Puglin => {
                if is_baby {
                    crate::sound::Sound::EntityWolfStep
                } else {
                    crate::sound::Sound::EntityWolfStep
                }
            }
            Self::Sad => {
                if is_baby {
                    crate::sound::Sound::EntityWolfStep
                } else {
                    crate::sound::Sound::EntityWolfStep
                }
            }
        }
    }
    #[must_use]
    pub const fn growl_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Angry => {
                if is_baby {
                    crate::sound::Sound::EntityWolfGrowl
                } else {
                    crate::sound::Sound::EntityWolfGrowl
                }
            }
            Self::Big => {
                if is_baby {
                    crate::sound::Sound::EntityWolfGrowl
                } else {
                    crate::sound::Sound::EntityWolfGrowl
                }
            }
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityWolfGrowl
                } else {
                    crate::sound::Sound::EntityWolfGrowl
                }
            }
            Self::Cute => {
                if is_baby {
                    crate::sound::Sound::EntityWolfGrowl
                } else {
                    crate::sound::Sound::EntityWolfGrowl
                }
            }
            Self::Grumpy => {
                if is_baby {
                    crate::sound::Sound::EntityWolfGrowl
                } else {
                    crate::sound::Sound::EntityWolfGrowl
                }
            }
            Self::Puglin => {
                if is_baby {
                    crate::sound::Sound::EntityWolfGrowl
                } else {
                    crate::sound::Sound::EntityWolfGrowl
                }
            }
            Self::Sad => {
                if is_baby {
                    crate::sound::Sound::EntityWolfGrowl
                } else {
                    crate::sound::Sound::EntityWolfGrowl
                }
            }
        }
    }
    #[must_use]
    pub const fn pant_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Angry => {
                if is_baby {
                    crate::sound::Sound::EntityWolfPant
                } else {
                    crate::sound::Sound::EntityWolfPant
                }
            }
            Self::Big => {
                if is_baby {
                    crate::sound::Sound::EntityWolfPant
                } else {
                    crate::sound::Sound::EntityWolfPant
                }
            }
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityWolfPant
                } else {
                    crate::sound::Sound::EntityWolfPant
                }
            }
            Self::Cute => {
                if is_baby {
                    crate::sound::Sound::EntityWolfPant
                } else {
                    crate::sound::Sound::EntityWolfPant
                }
            }
            Self::Grumpy => {
                if is_baby {
                    crate::sound::Sound::EntityWolfPant
                } else {
                    crate::sound::Sound::EntityWolfPant
                }
            }
            Self::Puglin => {
                if is_baby {
                    crate::sound::Sound::EntityWolfPant
                } else {
                    crate::sound::Sound::EntityWolfPant
                }
            }
            Self::Sad => {
                if is_baby {
                    crate::sound::Sound::EntityWolfPant
                } else {
                    crate::sound::Sound::EntityWolfPant
                }
            }
        }
    }
    #[must_use]
    pub const fn whine_sound(&self, is_baby: bool) -> crate::sound::Sound {
        match self {
            Self::Angry => {
                if is_baby {
                    crate::sound::Sound::EntityWolfWhine
                } else {
                    crate::sound::Sound::EntityWolfWhine
                }
            }
            Self::Big => {
                if is_baby {
                    crate::sound::Sound::EntityWolfWhine
                } else {
                    crate::sound::Sound::EntityWolfWhine
                }
            }
            Self::Classic => {
                if is_baby {
                    crate::sound::Sound::EntityWolfWhine
                } else {
                    crate::sound::Sound::EntityWolfWhine
                }
            }
            Self::Cute => {
                if is_baby {
                    crate::sound::Sound::EntityWolfWhine
                } else {
                    crate::sound::Sound::EntityWolfWhine
                }
            }
            Self::Grumpy => {
                if is_baby {
                    crate::sound::Sound::EntityWolfWhine
                } else {
                    crate::sound::Sound::EntityWolfWhine
                }
            }
            Self::Puglin => {
                if is_baby {
                    crate::sound::Sound::EntityWolfWhine
                } else {
                    crate::sound::Sound::EntityWolfWhine
                }
            }
            Self::Sad => {
                if is_baby {
                    crate::sound::Sound::EntityWolfWhine
                } else {
                    crate::sound::Sound::EntityWolfWhine
                }
            }
        }
    }
    #[must_use]
    pub const fn all() -> &'static [Self] {
        Self::ALL
    }
}
