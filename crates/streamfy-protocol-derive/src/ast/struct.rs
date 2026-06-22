use crate::ast::prop::{NamedProp, UnnamedProp};
use syn::{Fields, Generics, Ident, ItemStruct};

pub(crate) enum StreamfyStruct {
    Named(StreamfyNamedStruct),
    Tuple(StreamfyTupleStruct),
}

pub(crate) struct StreamfyNamedStruct {
    pub struct_ident: Ident,
    pub props: Vec<NamedProp>,
    generics: Generics,
}

impl StreamfyStruct {
    pub fn from_ast(item: &ItemStruct) -> syn::Result<Self> {
        let struct_ident = item.ident.clone();
        let generics = item.generics.clone();

        let streamfy_struct = match &item.fields {
            Fields::Named(fields) => {
                let mut props = vec![];
                for field in fields.named.iter() {
                    props.push(NamedProp::from_ast(field)?);
                }

                StreamfyStruct::Named(StreamfyNamedStruct {
                    struct_ident,
                    props,
                    generics,
                })
            }
            Fields::Unnamed(fields) => {
                let mut props = vec![];
                for field in fields.unnamed.iter() {
                    props.push(UnnamedProp::from_ast(field)?);
                }
                StreamfyStruct::Tuple(StreamfyTupleStruct {
                    struct_ident,
                    props,
                    generics,
                })
            }

            Fields::Unit => StreamfyStruct::Tuple(StreamfyTupleStruct {
                struct_ident,
                props: vec![],
                generics,
            }),
        };

        Ok(streamfy_struct)
    }

    pub fn struct_ident(&self) -> &Ident {
        match self {
            StreamfyStruct::Named(inner) => &inner.struct_ident,
            StreamfyStruct::Tuple(inner) => &inner.struct_ident,
        }
    }

    pub fn generics(&self) -> &Generics {
        match self {
            StreamfyStruct::Named(inner) => &inner.generics,
            StreamfyStruct::Tuple(inner) => &inner.generics,
        }
    }

    pub fn props(&self) -> StreamfyStructProps {
        match self {
            StreamfyStruct::Named(inner) => StreamfyStructProps::Named(inner.props.clone()),
            StreamfyStruct::Tuple(inner) => StreamfyStructProps::Unnamed(inner.props.clone()),
        }
    }
}

pub(crate) enum StreamfyStructProps {
    Named(Vec<NamedProp>),
    Unnamed(Vec<UnnamedProp>),
}

pub(crate) struct StreamfyTupleStruct {
    pub struct_ident: Ident,
    pub props: Vec<UnnamedProp>,
    pub generics: Generics,
}
