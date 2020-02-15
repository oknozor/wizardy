extern crate proc_macro;

mod gen;

use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields, Ident};

#[proc_macro_derive(Wizard)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = &input.ident;
    let builder_ident = format_ident!("{}{}", ident, "WizardBuilder");
    let wizard_ident = format_ident!("{}{}", ident, "Wizard");

    let fields: Vec<Field> = match input.data {
        Data::Struct(struct_) => match struct_.fields {
            Fields::Named(n) => n.named.iter().cloned().collect(),
            Fields::Unnamed(_) => panic!("Unexpected unnamed field"),
            Fields::Unit => panic!("Unexpected unit"),
        },
        Data::Enum(_) => panic!("The Builder macro is not to be used on enum"),
        Data::Union(_) => panic!("The Builder macro is not to be used on union"),
    };

    let field_idents: Vec<Ident> = gen::get_field_idents(fields);
    let ask_fn_idents: Vec<Ident> = field_idents
        .iter()
        .map(|ident| format_ident!("ask_{}", ident))
        .collect();

    let output = quote! {
        use std::io;
        use std::io::Write;
        use std::io::{stdin, Stdout};
        use std::process::exit;
        use termion::cursor::DetectCursorPos;
        use termion::event::Key;
        use termion::input::TermRead;
        use termion::raw::{IntoRawMode, RawTerminal};

        type Terminal = RawTerminal<Stdout>;

        pub struct #builder_ident {
            #(#field_idents: Option<String>,)*
        }

        impl #builder_ident {
            fn build(&self) -> #ident {
                #ident {
                    #(#field_idents: self.#field_idents.as_ref().expect("ee").clone(),)*
                }
            }

            #(pub fn #field_idents(&mut self, value: String) {
                self.#field_idents = Some(value);
            })*
        }

        pub struct #wizard_ident {
            terminal: Terminal,
            builder: #builder_ident,
            #(#field_idents: Option<String>,)*
        }

        impl #wizard_ident {
            pub fn new() -> Self {
                let terminal = io::stdout().into_raw_mode().unwrap();

                let builder = #builder_ident {
                    #(#field_idents : None,)*
                };

                #wizard_ident {
                    terminal,
                    builder,
                    #(#field_idents : None,)*
                }
            }

            #(pub fn #ask_fn_idents(&mut self, value: &str) -> &mut Self {
                self.#field_idents = Some(value.into());
                self
            })*

            pub fn run(&mut self) -> #ident {
            #(let #field_idents : Box<dyn Fn(String, &mut #builder_ident)> = Box::new(|value, builder| builder.#field_idents(value));)*

            let mut questions = vec![
                #((self.#field_idents.as_ref().expect("no #field_idents question"), #field_idents),)*
            ];

            while let Some((question, setter)) = questions.pop() {
                write!(self.terminal, "{}", termion::scroll::Up(1)).expect("Scroll error");
                let position = self.terminal.cursor_pos().expect("Can get cursor pos");

                write!(
                    self.terminal,
                    "{}{}",
                    termion::cursor::Goto(1, position.1),
                    question,
                )
                .unwrap();
                let mut output = String::new();

                for c in stdin().keys() {
                // Input
                    match c.expect("Key input error") {
                        Key::Char('\n') => break,
                        Key::Ctrl('c') => exit(0),
                        Key::Backspace => {
                            write!(
                                self.terminal,
                                "{} {}",
                                termion::cursor::Left(1),
                                termion::cursor::Left(1),
                            )
                                .expect("cannot go left");
                            let _ = output.pop();
                        }
                        Key::Char(c) => {
                            output.push(c);
                            write!(self.terminal, "{}", c).expect("Invalid char")
                        }
                        _ => (),
                    }
                    self.terminal.flush().expect("cannot flush");
                }
                setter(output.into(), &mut self.builder);
            }

            let position = self.terminal.cursor_pos().expect("Can get cursor pos");
            write!(self.terminal, "{}", termion::cursor::Goto(1, position.1 + 1)).expect("Scroll error");
            self.builder.build()

            }
        }

        impl #ident {
            pub fn wizard() -> #wizard_ident {
                #wizard_ident::new()
            }
        }
    };

    TokenStream::from(output)
}
