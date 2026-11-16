use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;


pub fn to_minuts_and_seconds(time_left: i64) -> (String, String){
    let minuts = time_left / 60;
    let seconds = time_left % 60;

    let minuts_display = if minuts < 10 { format!("0{minuts}")} else {
        format!("{minuts}")
    };
    let seconds_display = if seconds < 10 { format!("0{seconds}")} else {
        format!("{seconds}")
    };
    
    (minuts_display, seconds_display)
}


#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], catch)]
    pub async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"], js_name = "invoke", catch)]
    pub async fn invoke_no_args(cmd: &str) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["__TAURI__", "event"], js_name = listen)]
    pub async fn tauri_listen(event_name: &str, handler: &js_sys::Function) -> JsValue;
}