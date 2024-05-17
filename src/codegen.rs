use indoc::formatdoc;
use itertools::Itertools;

use crate::parser::{
    DeclaredType, Enum, EnumVariant, EnumVariantInner, InnerType, SerdeAttribute, Struct,
    StructField, Type,
};

fn indent(s: &str, level: usize) -> String {
    let indent = "    ".repeat(level);
    let sep = format!("\n{}", indent);
    s.lines().join(&sep)
}

trait StringExt {
    fn indent(&self, level: usize) -> String;
}

impl StringExt for str {
    fn indent(&self, level: usize) -> String {
        indent(self, level)
    }
}

fn support_struct(struc: &Struct) -> String {
    let skip_serializing: Vec<_> = struc
        .fields
        .iter()
        .filter(|field| {
            field
                .attrs
                .iter()
                .any(|attr| matches!(attr, SerdeAttribute::Skip | SerdeAttribute::SkipSerializing))
        })
        .collect();

    let skip_serializing = if skip_serializing.is_empty() {
        None
    } else {
        Some(
            formatdoc!(
                "
            SKIP_SERIALIZING = {{
                {}
            }}
            ",
                skip_serializing
                    .into_iter()
                    .map(|field| format!("\"{}\",", field.name))
                    .join("\n")
            )
            .indent(1),
        )
    };

    let skip_serializing_if: Vec<_> = struc
        .fields
        .iter()
        .filter_map(|field| {
            field.attrs.iter().find_map(|attr| match attr {
                SerdeAttribute::SkipSerializingIf(expr) => Some((field, expr)),
                _ => None,
            })
        })
        .collect();

    let skip_serializing_if = if skip_serializing_if.is_empty() {
        None
    } else {
        Some(
            formatdoc!(
                "
            SKIP_SERIALIZING_IF = {{
                {}
            }}
            ",
                skip_serializing_if
                    .into_iter()
                    .map(|(field, expr)| format!("\"{}\": {},", field.name, expr))
                    .join("\n")
            )
            .indent(1),
        )
    };

    let skip_deserializing: Vec<_> = struc
        .fields
        .iter()
        .filter(|field| {
            field.attrs.iter().any(|attr| {
                matches!(
                    attr,
                    SerdeAttribute::Skip | SerdeAttribute::SkipDeserializing
                )
            })
        })
        .collect();

    let skip_deserializing = if skip_deserializing.is_empty() {
        None
    } else {
        Some(
            formatdoc!(
                "
            SKIP_DESERIALIZING = {{
                {}
            }}
            ",
                skip_deserializing
                    .into_iter()
                    .map(|field| format!("\"{}\",", field.name))
                    .join("\n")
            )
            .indent(1),
        )
    };

    let skip_deserializing_if: Vec<_> = struc
        .fields
        .iter()
        .filter_map(|field| {
            field.attrs.iter().find_map(|attr| match attr {
                SerdeAttribute::SkipDeserializingIf(expr) => Some((field, expr)),
                _ => None,
            })
        })
        .collect();

    let skip_deserializing_if = if skip_deserializing_if.is_empty() {
        None
    } else {
        Some(
            formatdoc!(
                "
            SKIP_DESERIALIZING_IF = {{
                {}
            }}
            ",
                skip_deserializing_if
                    .into_iter()
                    .map(|(field, expr)| format!("\"{}\": {},", field.name, expr))
                    .join("\n")
                    .indent(1)
            )
            .indent(1),
        )
    };

    let rename = struc
        .fields
        .iter()
        .filter_map(|field| {
            field.attrs.iter().find_map(|attr| match attr {
                SerdeAttribute::Rename(name) => Some((field, name)),
                _ => None,
            })
        })
        .collect_vec();

    let rename = if rename.is_empty() {
        None
    } else {
        Some(
            formatdoc!(
                "
            RENAME = {{
                {}
            }}
            ",
                rename
                    .into_iter()
                    .map(|(field, name)| format!("\"{}\": \"{}\",", field.name, name))
                    .join("\n")
            )
            .indent(1),
        )
    };

    let mut support = String::new();

    if let Some(skip_serializing) = skip_serializing {
        support.push_str(&skip_serializing);
    }

    if let Some(skip_serializing_if) = skip_serializing_if {
        support.push_str(&skip_serializing_if);
    }

    if let Some(skip_deserializing) = skip_deserializing {
        support.push_str(&skip_deserializing);
    }

    if let Some(skip_deserializing_if) = skip_deserializing_if {
        support.push_str(&skip_deserializing_if);
    }

    if let Some(rename) = rename {
        support.push_str(&rename);
    }

    support.push('\n');

    support
}

macro_rules! output {
    ($target:expr, $($arg:tt)*) => {
        $target.write(&format!($($arg)*));
    };
}

pub struct Codegen<'a> {
    types: Vec<DeclaredType<'a>>,

    output: String,
}

impl<'a> Codegen<'a> {
    pub fn new(types: Vec<DeclaredType<'a>>) -> Self {
        Codegen {
            types,
            output: String::new(),
        }
    }

    fn write(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn generate_type(&self, type_: &Type<'a>) -> String {
        match type_ {
            Type::String => "str".to_string(),
            Type::Int => "int".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Unit => "None".to_string(),
            Type::Float => "float".to_string(),
            Type::Tuple(types) => {
                let types: Vec<_> = types.iter().map(|t| self.generate_type(t)).collect();
                format!("tuple[{}]", types.join(", "))
            }
            Type::Vec(ty) => format!("list[{}]", self.generate_type(ty)),
            Type::Array(ty) => format!("list[{}]", self.generate_type(ty)),
            Type::User(name) => name.to_string(),
            Type::Option(ty) => format!("{} | None", self.generate_type(&*ty)),
        }
    }

    fn generate_field(&mut self, field: &StructField<'a>) {
        let tyname = self.generate_type(&field.ty);

        let mut default = None;

        for attr in &field.attrs {
            match attr {
                SerdeAttribute::Default(val) => {
                    if let Some(val) = val {
                        default = Some(*val);
                    } else {
                        default = match &field.ty {
                            Type::String => Some("\"\""),
                            Type::Int => Some("0"),
                            Type::Bool => Some("False"),
                            Type::Unit => Some("None"),
                            Type::Float => Some("0.0"),
                            Type::Tuple(_) => Some("()"),
                            Type::Vec(_) => Some("[]"),
                            Type::Array(_) => Some("[]"),
                            Type::Option(_) => Some("None"),
                            Type::User(_) => todo!("default for user type"),
                        };
                    }
                }
                _ => {} // ignore other attributes for now,
            }
        }

        if let Some(default) = default {
            output!(self, "    {}: {} = {}\n", field.name, tyname, default);
        } else {
            output!(self, "    {}: {}\n", field.name, tyname);
        }
    }

    fn generate_struct(&mut self, name: &str, struc: &Struct<'a>) {
        output!(self, "@dataclass\n");
        output!(self, "class {}:\n", name);
        output!(self, "    {}\n", support_struct(struc));

        let fields = struc.fields.iter().sorted_by_key(|field| {
            field
                .attrs
                .iter()
                .any(|attr| matches!(attr, SerdeAttribute::Default(..)))
        });

        for field in fields {
            self.generate_field(&field);
        }
    }

    fn generate_enum_tuple(
        &mut self,
        name: &str,
        _attrs: &[SerdeAttribute<'a>],
        types: &[Type<'a>],
    ) {
        output!(self, "@dataclass\n");
        output!(self, "class {}(TupleVariant):\n", name);
        output!(
            self,
            "    ENUM_DATA = (ENUM_VARIANT_TUPLE, \"{}\")\n\n",
            Self::safe_name(name)
        );

        for (i, ty) in types.iter().enumerate() {
            let tyname = self.generate_type(ty);
            output!(self, "    _{}: {}\n", i, tyname);
        }

        output!(self, "\n");
    }

    fn generate_enum_struct(
        &mut self,
        name: &str,
        _attrs: &[SerdeAttribute<'a>],
        struc: &Struct<'a>,
    ) {
        output!(self, "@dataclass\n");
        output!(self, "class {}:\n", name);
        output!(
            self,
            "    ENUM_DATA = (ENUM_VARIANT_STRUCT, \"{}\")\n",
            Self::safe_name(name)
        );
        output!(self, "    {}\n", support_struct(struc));

        for field in &struc.fields {
            self.generate_field(&field);
        }

        output!(self, "\n");
    }

    fn safe_name(name: &str) -> &str {
        if name == "type" {
            "type_" // type is a reserved keyword
        } else if name == "None" {
            "None_" // None is a reserved keyword
        } else {
            name // otherwise, use the name as is
        }
    }

    fn generate_enum_variant(&mut self, variant: &EnumVariant<'a>) {
        let name = &variant.name;

        match &variant.inner {
            EnumVariantInner::Unit => {
                output!(self, "@dataclass\n");
                output!(self, "class {}:\n", Self::safe_name(name));
                output!(
                    self,
                    "    ENUM_DATA = (ENUM_VARIANT_UNIT, \"{}\")\n\n",
                    variant.name
                );
            }
            EnumVariantInner::Tuple(types) => self.generate_enum_tuple(name, &variant.attrs, types),
            EnumVariantInner::Struct(struc) => {
                self.generate_enum_struct(name, &variant.attrs, struc)
            }
        }
    }

    fn generate_enum(&mut self, name: &str, enum_: &Enum<'a>) {
        for variant in &enum_.variants {
            self.generate_enum_variant(variant);
        }

        let variants = enum_
            .variants
            .iter()
            .map(|variant| Self::safe_name(variant.name))
            .join(" | ");

        output!(self, "{name} = {variants}\n");
    }

    fn generate_decl_type(&mut self, type_: &DeclaredType<'a>) {
        let name = Self::safe_name(&type_.name);

        match &type_.inner {
            InnerType::Struct(struc) => self.generate_struct(name, struc),
            InnerType::Enum(enum_) => self.generate_enum(name, enum_),
        }

        output!(self, "\n\n");
    }

    pub fn generate(mut self) -> String {
        output!(self, "# Generated code\n\n");
        output!(
            self,
            "from lib.marshal import TupleVariant, dataclass, asdict\n\n"
        );

        let types = std::mem::take(&mut self.types);

        for type_ in &types {
            self.generate_decl_type(type_);
        }

        self.output
    }
}
