// 基础实体组件库
// 原型可复用组件：血量、攻击/受击判定、属性集、输入缓冲、射击、子弹、近战

mod health;
mod combat;
mod attribute;
mod input_buffer;
mod bullet;
mod shooter;
mod melee;

pub use health::*;
pub use combat::*;
pub use attribute::*;
pub use input_buffer::*;
pub use bullet::*;
pub use shooter::*;
pub use melee::*;
