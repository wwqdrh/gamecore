// GdJsonCodec - GJson 加密编解码的 Godot 静态方法接口
//
// 用途：静态定义表管线（明文 .json 源 → 加密 .gjson 产物）。
//   - 编辑器插件/无头工具读取明文 .json 后调用 encrypt_text 生成 .gjson 产物
//   - Table 查询类运行时读取 .gjson 产物后调用 decrypt_to_text 还原再解析
// 密钥单一来源：GJson::ENCRYPT_KEY（与 GdCoreData 运行时存档同一套加密），
// GDScript 侧永远不接触密钥，只通过本类做字节级编解码。

use godot::prelude::*;

use super::gjson::GJson;

#[derive(GodotClass)]
#[class(base = RefCounted)]
pub struct GdJsonCodec {
    base: Base<RefCounted>,
}

#[godot_api]
impl IRefCounted for GdJsonCodec {
    fn init(base: Base<RefCounted>) -> Self {
        Self { base }
    }
}

#[godot_api]
impl GdJsonCodec {
    /// 明文 JSON 文本 → 加密字节流（供 FileAccess.store_buffer 写 .gjson 产物）
    #[func]
    pub fn encrypt_text(text: GString) -> PackedByteArray {
        PackedByteArray::from(GJson::encrypt(&text.to_string()).as_slice())
    }

    /// 加密字节流 → 明文 JSON 文本（供 JSON.parse_string 解析）
    #[func]
    pub fn decrypt_to_text(data: PackedByteArray) -> GString {
        GString::from(GJson::decrypt(&data.to_vec()).as_str())
    }
}
