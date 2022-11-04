use crate::runtime::access_util::access_type::AccessType;

pub struct AccessSpecification {
    access_type: AccessType,
    location_indicator: Option<usize>,
    // if access_type == Stack -> location = depth - location_indicator
    // if access_type == Memory -> location depending on the stack
}

impl AccessSpecification {
    pub fn new(access_type: AccessType, location_indicator: Option<usize>) -> Self {
        match access_type {
            AccessType::Garbage => {
                assert!(location_indicator == None);
            },
            AccessType::Stack => {
                assert!(location_indicator != None);
            },
            AccessType::Memory => {
                assert!(location_indicator == None);
            },
        }

        Self {
            access_type: access_type,
            location_indicator: location_indicator,
        }
    }
}