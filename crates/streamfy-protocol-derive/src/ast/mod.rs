pub(crate) mod container;
pub(crate) mod r#enum;
pub(crate) mod prop;
pub(crate) mod r#struct;

use syn::parse::{Parse, ParseStream};
use syn::{
    Attribute, GenericParam, Generics, ItemEnum, ItemStruct, Result, Token, Visibility, parse_quote,
};

use crate::ast::container::ContainerAttributes;
use crate::ast::r#enum::StreamfyEnum;
use crate::ast::r#struct::StreamfyStruct;

pub(crate) enum DeriveItem {
    Struct(StreamfyStruct, ContainerAttributes),
    Enum(StreamfyEnum, ContainerAttributes),
}

impl Parse for DeriveItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = ContainerAttributes::from_ast(&input.call(Attribute::parse_outer)?)?;
        let _vis = input.parse::<Visibility>()?;

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![struct]) {
            let item_struct: ItemStruct = input.parse()?;
            let kf_struct = StreamfyStruct::from_ast(&item_struct)?;
            Ok(DeriveItem::Struct(kf_struct, attrs))
        } else if lookahead.peek(Token![enum]) {
            let item_enum: ItemEnum = input.parse()?;
            let kf_enum = StreamfyEnum::from_ast(item_enum, &attrs)?;
            Ok(DeriveItem::Enum(kf_enum, attrs))
        } else {
            Err(lookahead.error())
        }
    }
}

pub(crate) enum StreamfyBound {
    Encoder,
    Decoder,
    Default,
}

pub(crate) fn add_bounds(
    mut generics: Generics,
    attr: &ContainerAttributes,
    bounds: StreamfyBound,
) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            match bounds {
                StreamfyBound::Encoder => {
                    type_param
                        .bounds
                        .push(parse_quote!(streamfy_protocol::Encoder));
                }
                StreamfyBound::Decoder => {
                    type_param
                        .bounds
                        .push(parse_quote!(streamfy_protocol::Decoder));
                }
                StreamfyBound::Default => {
                    type_param.bounds.push(parse_quote!(Default));
                }
            }
            if attr.trace {
                type_param.bounds.push(parse_quote!(std::fmt::Debug));
            }
        }
    }

    generics
}
