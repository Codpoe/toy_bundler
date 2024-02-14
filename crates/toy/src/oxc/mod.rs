use ouroboros::self_referencing;
use oxc::{
  allocator::Allocator,
  ast::{ast::Program, AstBuilder},
  parser::Parser,
  span::SourceType,
};

pub struct OxcAllocatorWrapper(pub Allocator);

unsafe impl Send for OxcAllocatorWrapper {}
unsafe impl Sync for OxcAllocatorWrapper {}

#[derive(Debug)]
pub struct OxcProgramWrapper<'a>(pub Program<'a>);

unsafe impl<'a> Send for OxcProgramWrapper<'a> {}
unsafe impl<'a> Sync for OxcProgramWrapper<'a> {}

impl<'a> OxcProgramWrapper<'a> {
  pub fn new(
    allocator: &'a OxcAllocatorWrapper,
    source_text: &'a str,
    source_type: SourceType,
  ) -> Self {
    let ret = Parser::new(&allocator.0, source_text, source_type).parse();
    Self(ret.program)
  }
}

pub struct OxcAstBuilderWrapper<'a>(pub AstBuilder<'a>);

unsafe impl<'a> Send for OxcAstBuilderWrapper<'a> {}
unsafe impl<'a> Sync for OxcAstBuilderWrapper<'a> {}

#[self_referencing]
pub struct OxcProgram {
  pub allocator: OxcAllocatorWrapper,
  source_text: String,
  source_type: SourceType,
  #[borrows(allocator, source_text, source_type)]
  #[not_covariant]
  pub program: OxcProgramWrapper<'this>,
}

impl OxcProgram {
  pub fn build(source_text: String, source_type: SourceType) -> Self {
    OxcProgramBuilder {
      allocator: OxcAllocatorWrapper(Allocator::default()),
      source_text,
      source_type,
      program_builder: |allocator, source_text, source_type| {
        let ret = Parser::new(&allocator.0, source_text, source_type.clone()).parse();
        OxcProgramWrapper(ret.program)
      },
    }
    .build()
  }

  pub fn copy_program(&self) -> Program {
    self.with_program(|program_wrapper| unsafe { std::mem::transmute_copy(&program_wrapper.0) })
  }
}

impl std::fmt::Debug for OxcProgram {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("OxcProgram")
      .field("source_text", self.borrow_source_text())
      .field("source_type", self.borrow_source_type())
      .finish()
  }
}

#[self_referencing]
pub struct OxcAstBuilder {
  pub allocator: OxcAllocatorWrapper,
  #[borrows(allocator)]
  #[not_covariant]
  pub builder_wrapper: OxcAstBuilderWrapper<'this>,
}

impl OxcAstBuilder {
  pub fn build() -> Self {
    OxcAstBuilderBuilder {
      allocator: OxcAllocatorWrapper(Allocator::default()),
      builder_wrapper_builder: |allocator| OxcAstBuilderWrapper(AstBuilder::new(&allocator.0)),
    }
    .build()
  }

  pub fn get_ast_builder(&self) -> &AstBuilder {
    self.with_builder_wrapper(|builder_wrapper| &builder_wrapper.0)
  }
}
