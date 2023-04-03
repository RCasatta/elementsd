pub const HAS_FEATURE: bool = cfg!(any(
    feature = "22_1_1",
    feature = "0_21_0",
    feature = "0_18_1_12",
));

#[cfg(not(any(
    feature = "22_1_1",
    feature = "0_21_0",
    feature = "0_18_1_12",
)))]
pub const VERSION: &str = "N/A";

#[cfg(feature = "22_1_1")]
pub const VERSION: &str = "22.1.1";

#[cfg(feature = "0_21_0")]
pub const VERSION: &str = "elements-0.21.0";

#[cfg(feature = "0_18_1_12")]
pub const VERSION: &str = "0.18.1.12";
