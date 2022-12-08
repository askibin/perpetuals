// admin instructions
// pub mod init;
// pub mod set_admin_signers;
// pub mod set_fees;
// pub mod set_permissions;
// pub mod withdraw_fees;

// test instructions
// pub mod set_test_oracle_price;
// pub mod set_test_time;
pub mod test_init;

// public instructions
pub mod close_position;
pub mod open_position;

// bring everything in scope
// pub use init::*;
// pub use set_admin_signers::*;
// pub use set_fees::*;
// pub use set_permissions::*;
// pub use withdraw_fees::*;

// pub use set_test_oracle_price::*;
// pub use set_test_time::*;
pub use test_init::*;

pub use close_position::*;
pub use open_position::*;
