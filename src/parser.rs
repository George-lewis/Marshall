

use crate::lexer::{LexicalToken, TokenType};

#[derive(Debug)]
pub enum Type<'a> {
    Unit,
    Bool,
    Int,
    Float,
    String,
    Option(Box<Type<'a>>),
    Tuple(Vec<Type<'a>>),
    Array(Box<Type<'a>>),
    Vec(Box<Type<'a>>),
    // Struct(Rc<Struct<'a>>),
    // Enum(Rc<Enum<'a>>),
    User(&'a str),
}

#[derive(Debug)]
pub struct StructField<'a> {
    pub attrs: Vec<SerdeAttribute<'a>>,
    pub name: &'a str,
    pub ty: Type<'a>,
}

#[derive(Debug)]
pub struct Struct<'a> {
    pub fields: Vec<StructField<'a>>,
}

#[derive(Debug)]
pub enum EnumVariantInner<'a> {
    Unit,
    Tuple(Vec<Type<'a>>),
    Struct(Struct<'a>),
}

#[derive(Debug)]
pub enum SerdeAttribute<'a> {
    Rename(&'a str),
    Default(Option<&'a str>),
    Skip,
    SkipSerializing,
    SkipDeserializing,
    SkipSerializingIf(&'a str),
    SkipDeserializingIf(&'a str),
    SerializeWith(&'a str),
    DeserializeWith(&'a str),
}

#[derive(Debug)]
pub struct EnumVariant<'a> {
    pub attrs: Vec<SerdeAttribute<'a>>,
    pub name: &'a str,
    pub inner: EnumVariantInner<'a>,
}

#[derive(Debug)]
pub struct Enum<'a> {
    pub variants: Vec<EnumVariant<'a>>,
}

#[derive(Debug)]
pub enum InnerType<'a> {
    Struct(Struct<'a>),
    Enum(Enum<'a>),
}

#[derive(Debug)]
pub struct DeclaredType<'a> {
    pub name: &'a str,
    pub inner: InnerType<'a>,
}

pub struct Parser<'a> {
    pub tokens: &'a [LexicalToken<'a>],
    pub types: Vec<DeclaredType<'a>>,
    pub cursor: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [LexicalToken<'a>]) -> Self {
        Parser {
            tokens,
            types: Vec::new(),
            cursor: 0,
        }
    }

    pub fn parse(mut self) -> Vec<DeclaredType<'a>> {
        while self.cursor < self.tokens.len() {
            self.parse_decl();
        }

        self.types
    }

    fn parse_decl(&mut self) {
        if let TokenType::Hash = self.tokens[self.cursor].token {
            self.cursor += 1;

            while !matches!(
                self.tokens[self.cursor].token,
                TokenType::Struct | TokenType::Enum
            ) {
                self.cursor += 1;
            }
        }

        let lex = &self.tokens[self.cursor];
        self.cursor += 1;

        match &lex.token {
            TokenType::Enum => self.parse_enum(),
            TokenType::Struct => self.parse_struct(),
            ty => panic!("unexpected token {:?}; expected struct or enum", ty),
        }
    }

    fn get_ident(&mut self) -> &'a str {
        match &self.tokens[self.cursor].token {
            TokenType::Identifier(name) => name,
            ty => panic!("unexpected token {:?}; expected identifier", ty),
        }
    }

    fn eat_ident(&mut self) -> &'a str {
        let name = self.get_ident();
        self.cursor += 1;
        name
    }

    fn eat_string(&mut self) -> &'a str {
        let name = self.get_string();
        self.cursor += 1;
        name
    }

    fn get_string(&mut self) -> &'a str {
        match &self.tokens[self.cursor].token {
            TokenType::String(name) => name,
            ty => panic!("unexpected token {:?}; expected string", ty),
        }
    }

    fn parse_variant(&mut self) -> EnumVariant<'a> {
        let attrs = self.parse_serde_attributes();
        let name = self.eat_ident();

        let inner = if self.eat(&TokenType::LParen) {
            let types = self.parse_tuple_inner();

            EnumVariantInner::Tuple(types)
        } else if self.eat(&TokenType::LBrace) {
            let fields = self.parse_struct_fields();

            EnumVariantInner::Struct(Struct { fields })
        } else {
            EnumVariantInner::Unit
        };

        EnumVariant { attrs, name, inner }
    }

    fn parse_enum(&mut self) {
        let name = self.eat_ident();

        let mut fields = Vec::new();

        self.must_eat(TokenType::LBrace);

        loop {
            use TokenType::*;
            let f = self.parse_variant();

            fields.push(f);

            if self.eat(&RBrace) {
                break;
            }

            if self.eat(&Comma) {
                if self.eat(&RBrace) {
                    break;
                }

                // continue
            } else {
                panic!("unexpected token {:?}", self.tokens[self.cursor].token);
            }
        }

        let en = DeclaredType {
            name,
            inner: InnerType::Enum(Enum { variants: fields }),
        };

        self.types.push(en);
    }

    // eat { before calling
    fn parse_struct_fields(&mut self) -> Vec<StructField<'a>> {
        let mut fields = Vec::new();

        loop {
            use TokenType::*;
            let f = self.parse_field();

            fields.push(f);

            if self.eat(&Comma) {
                if self.eat(&RBrace) {
                    break;
                }

                // continue
            } else if self.eat(&RBrace) {
                break;
            } else {
                panic!("unexpected token {:?}", self.tokens[self.cursor].token);
            }
        }

        fields
    }

    fn parse_struct(&mut self) {
        let name = self.eat_ident();

        self.must_eat(TokenType::LBrace);

        let fields = self.parse_struct_fields();

        let struc = DeclaredType {
            name,
            inner: InnerType::Struct(Struct { fields }),
        };

        self.types.push(struc);
    }

    fn parse_field(&mut self) -> StructField<'a> {
        use TokenType::*;

        let attrs = self.parse_serde_attributes();

        // may or may not be present
        // we don't care about it
        self.eat(&Pub);

        let name = self.eat_ident();

        self.must_eat(Colon);

        let ty = self.parse_type();

        StructField { attrs, name, ty }
    }

    // eat ( before calling
    fn parse_tuple(&mut self) -> Type<'a> {
        let types = self.parse_tuple_inner();

        Type::Tuple(types)
    }

    // eat ( before calling
    fn parse_tuple_inner(&mut self) -> Vec<Type<'a>> {
        use TokenType::*;

        let mut types = Vec::new();

        loop {
            types.push(self.parse_type());

            if self.eat(&RParen) {
                break;
            }

            self.must_eat(Comma);
        }

        types
    }

    fn parse_type(&mut self) -> Type<'a> {
        use TokenType::*;

        if self.eat(&LParen) {
            if self.eat(&RParen) {
                return Type::Unit;
            }

            self.parse_tuple()
        } else {
            // all other types start with an ident
            let ident = self.eat_ident();

            match ident {
                "bool" => return Type::Bool,
                "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" => return Type::Int,
                "f32" | "f64" => return Type::Float,
                "String" => return Type::String,
                "Vec" => {
                    self.must_eat(Langle);
                    let ty = Box::new(self.parse_type());
                    self.must_eat(Rangle);
                    return Type::Vec(ty);
                }
                "Array" => {
                    self.must_eat(LBracket);
                    let ty = Box::new(self.parse_type());
                    self.must_eat(RBracket);
                    return Type::Array(ty);
                }
                "Option" => {
                    self.must_eat(Langle);
                    let ty = Box::new(self.parse_type());
                    self.must_eat(Rangle);
                    return Type::Option(ty);
                }
                _ => return Type::User(ident),
            }
        }
    }

    fn peek(&self, tok: TokenType<'a>) -> bool {
        self.tokens[self.cursor].token == tok
    }

    fn eat(&mut self, tok: &TokenType<'a>) -> bool {
        if self.tokens[self.cursor].token == *tok {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn must_eat(&mut self, tok: TokenType<'a>) {
        if !self.eat(&tok) {
            panic!(
                "unexpected token {:?}; expected {:?}",
                self.tokens[self.cursor].token, tok
            );
        }
    }

    fn parse_serde_attributes(&mut self) -> Vec<SerdeAttribute<'a>> {
        use TokenType::*;

        let mut attrs = Vec::new();

        if self.eat(&Hash) {
            self.must_eat(LBracket);
            self.must_eat(Identifier("serde"));
            self.must_eat(LParen);

            loop {
                use SerdeAttribute::*;

                let name = self.eat_ident();

                if name == "skip" {
                    attrs.push(Skip);
                } else if name == "skip_serializing" {
                    attrs.push(SkipSerializing);
                } else if name == "skip_deserializing" {
                    attrs.push(SkipDeserializing);
                } else if name == "skip_serializing_if" {
                    self.must_eat(Equals);
                    let cond = self.eat_string();
                    attrs.push(SkipSerializingIf(cond));
                } else if name == "skip_deserializing_if" {
                    self.must_eat(Equals);
                    let cond = self.eat_string();
                    attrs.push(SkipDeserializingIf(cond));
                } else if name == "serialize_with" {
                    self.must_eat(Equals);
                    let cond = self.eat_string();
                    attrs.push(SerializeWith(cond));
                } else if name == "deserialize_with" {
                    self.must_eat(Equals);
                    let cond = &self.eat_string();
                    attrs.push(DeserializeWith(cond));
                } else if name == "rename" {
                    self.must_eat(Equals);
                    let cond = self.eat_string();
                    attrs.push(Rename(cond));
                } else if name == "default" {
                    if self.eat(&Equals) {
                        let cond = self.eat_string();

                        attrs.push(Default(Some(cond)));
                    } else {
                        attrs.push(Default(None));
                    }
                } else {
                    panic!("unexpected serde attribute {:?}", name);
                }

                if self.eat(&RParen) {
                    self.must_eat(RBracket);

                    break;
                }

                self.must_eat(Comma);
            }
        }

        attrs
    }
}
