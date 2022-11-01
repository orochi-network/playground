use crate::runtime::access_util::access_type::AccessType;

pub struct AccessSpecification {
    access_type: AccessType,
    location_indicator: usize,
    // if access_type == Stack -> location = depth - location_indicator
    // if access_type == Memory -> location = location_indicator
}