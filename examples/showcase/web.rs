use eframe::web_sys::HtmlCanvasElement;
use wasm_bindgen::prelude::*;

use super::Showcase;

#[wasm_bindgen]
pub struct WebShowcase {
    runner: eframe::WebRunner,
}

#[wasm_bindgen]
impl WebShowcase {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            runner: eframe::WebRunner::new(),
        }
    }

    pub async fn start(
        &self,
        canvas: HtmlCanvasElement,
        reduced_motion: bool,
        touch: bool,
    ) -> Result<(), JsValue> {
        self.runner
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(move |_| {
                    Ok(Box::new(Showcase {
                        reduced_motion,
                        web_touch: touch,
                        ..Showcase::default()
                    }))
                }),
            )
            .await
    }
}
