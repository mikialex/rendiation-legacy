pub struct CodeBuilder {
  tab: String,
  tab_state: usize,
  str: String,
}

impl Default for CodeBuilder {
  fn default() -> Self {
    Self {
      tab: String::from("  "),
      tab_state: 0,
      str: String::new(),
    }
  }
}

impl CodeBuilder {
  pub fn tab(&mut self) -> &mut Self {
    self.tab_state += 1;
    self
  }
  pub fn un_tab(&mut self) -> &mut Self {
    self.tab_state -= 1;
    self
  }
  pub fn write_ln(&mut self, content: impl AsRef<str>) -> &mut Self {
    self.str.push('\n');
    (0..self.tab_state).for_each(|_| self.str.push_str(&self.tab));
    self.str.push_str(content.as_ref());
    self
  }

  pub fn write_raw(&mut self, content: impl AsRef<str>) -> &mut Self {
    self.str.push_str(content.as_ref());
    self
  }

  pub fn output(self) -> String {
    self.str
  }
}
