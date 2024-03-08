use std::ops::Deref;

use lightningcss::{
  rules::CssRule,
  stylesheet::{ParserOptions, StyleSheet},
};
use ouroboros::self_referencing;

#[self_referencing]
pub struct LightningStyleSheet {
  code: String,
  #[borrows(code)]
  #[not_covariant]
  pub style_sheet: StyleSheet<'this, 'this>,
}

impl LightningStyleSheet {
  pub fn build(code: String, filename: String) -> LightningStyleSheet {
    LightningStyleSheetBuilder {
      code,
      style_sheet_builder: |code| {
        StyleSheet::parse(
          code,
          ParserOptions {
            filename,
            ..ParserOptions::default()
          },
        )
        .unwrap()
      },
    }
    .build()
  }

  pub fn copy_css_rules(&self) -> Vec<CssRule> {
    self.with_style_sheet(|style_sheet| {
      let mut css_rules = vec![];

      for rule in &style_sheet.rules.0 {
        css_rules.push(unsafe { std::mem::transmute_copy(rule) });
      }

      css_rules
    })
  }
}

impl std::fmt::Debug for LightningStyleSheet {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.with_style_sheet(|style_sheet| {
      f.debug_struct("LightningStyleSheet")
        .field("code", self.borrow_code())
        .field("style_sheet", style_sheet)
        .finish()
    })
  }
}
