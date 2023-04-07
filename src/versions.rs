#[cfg(feature = "22_1_1")]
pub const VERSION: &str = "22.1.1";

#[cfg(all(feature = "0_21_0", not(feature = "22_1_1")))]
pub const VERSION: &str = "elements-0.21.0";

#[cfg(all(feature = "0_18_1_12", not(feature = "0_21_0")))]
pub const VERSION: &str = "0.18.1.12";

#[cfg(not(feature = "0_18_1_12"))]
pub const VERSION: &str = "N/A";
