//! 下游客户端模型请求正文的 Codex 协议兼容。

use serde_json::{Map, Value};

/// 补齐 Codex 请求缺省字段并移除已确认不适用的顶层参数，不递归清洗业务正文。
///
/// 兼容基准是 Codex Core/Desktop 的模型请求，不是公开 OpenAI Responses API。
/// 未知字段继续透传，不能因官方请求结构中没有某个字段就将其列入过滤规则。
pub(in crate::transport) fn normalize_codex_request_body(body: &mut Map<String, Value>) {
    // 官方 Core/Desktop 显式发送 store=false；仅为缺字段的下游请求补齐，保留显式值。
    body.entry("store").or_insert(Value::Bool(false));

    // Codex Responses 不接受 input 中的 system role；developer role 保留同等的
    // 高优先级指令语义，同时兼容通用 Responses 客户端生成的 message。
    if let Some(input) = body.get_mut("input").and_then(Value::as_array_mut) {
        for item in input {
            let Some(item) = item.as_object_mut() else {
                continue;
            };
            if item.get("type").and_then(Value::as_str) == Some("message")
                && item.get("role").and_then(Value::as_str) == Some("system")
            {
                item.insert("role".to_owned(), Value::String("developer".to_owned()));
            }
        }
    }

    for field in [
        // Pi 普通 Responses 适配将 maxTokens 映射为 max_output_tokens，
        // temperature 则原样写入；Pi 的 Codex 适配也可能发送 temperature。
        "max_output_tokens",
        "temperature",
        // Pi 开启长缓存时发送 24h；保留有效的 prompt_cache_key，
        // 只剥离 Codex Responses 明确拒绝的缓存保留时长参数。
        "prompt_cache_retention",
    ] {
        body.remove(field);
    }
}
