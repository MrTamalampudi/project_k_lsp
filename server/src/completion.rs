use std::cmp::Ordering;

use project_k::{
    enums::{Browser, Capabilities},
    keywords::TokenType,
    token::Token,
};
use tower_lsp::lsp_types::{CompletionItem, Position};

pub struct IntelliSense {
    tokens: Vec<Token>,
    position: Position,
}

impl IntelliSense {
    pub fn new(tokens: &Vec<Token>, position: &Position) -> IntelliSense {
        IntelliSense {
            tokens: tokens.clone(),
            position: position.clone(),
        }
    }

    pub fn complete(&self) -> Vec<CompletionItem> {
        let (index, filetype, highlevel) = self.get_highlevel_and_filetype_token();
        match highlevel {
            TokenType::CAPABILITIES => self.complete_capabilities(index),
            TokenType::TESTSTEPS => self.complete_teststeps(index),
            _ => todo!(),
        }
    }

    fn complete_teststeps(&self, index: usize) -> Vec<CompletionItem> {
        todo!()
    }

    fn complete_capabilities(&self, index: usize) -> Vec<CompletionItem> {
        let token_type = self.tokens.get(index - 1).unwrap().get_token_type();
        if token_type == TokenType::NEW_LINE {
            complete_capbility_key()
        } else if token_type == TokenType::ASSIGN_OP {
            let token = self.tokens.get(index - 2).unwrap().get_token_type();
            complete_capability_value(token)
        } else {
            complete_capbility_key()
        }
    }

    fn get_testcase_high_level_token(&self, index: usize) -> TokenType {
        let mut index = index;
        let mut token_type = TokenType::NONE;
        while index > 0 && token_type == TokenType::NONE {
            let token = match self.tokens.get(index) {
                Some(t) => t,
                None => break,
            };
            match token.get_token_type() {
                TokenType::TESTSTEPS | TokenType::CAPABILITIES | TokenType::PREREQUISITE => {
                    token_type = token.get_token_type();
                }
                _ => {}
            };

            index -= 1;
        }
        token_type
    }

    fn get_highlevel_token(&self, filetype: TokenType, index: usize) -> TokenType {
        match filetype {
            TokenType::TESTCASE => self.get_testcase_high_level_token(index),
            _ => TokenType::NONE,
        }
    }

    fn token_comparator(&self, token: &Token) -> Ordering {
        //subtracting 1 from line beacause lsp range starts from 0
        let token_end_line = (token.get_end_location().line - 1) as u32;
        //for this some how it worked
        let token_end_column = (token.get_end_location().column) as u32;
        //check Ordering struct how less & greater have been decided
        if token_end_line == self.position.line {
            if token_end_column == self.position.character {
                Ordering::Equal
            } else if token_end_column < self.position.character {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        } else if token_end_line < self.position.line {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }

    pub fn get_highlevel_and_filetype_token(&self) -> (usize, TokenType, TokenType) {
        let index: usize = match self
            .tokens
            .binary_search_by(|token| self.token_comparator(token))
        {
            Ok(index) => index,
            Err(index) => index,
        };

        let filetype = self.tokens.get(0).unwrap();
        let highlevel_token = self.get_highlevel_token(filetype.get_token_type(), index);

        (index, filetype.get_token_type(), highlevel_token)
    }
}

//here token is capability key token
fn complete_capability_value(token: TokenType) -> Vec<CompletionItem> {
    let string = match token {
        TokenType::IDENTIFIER(string) => string,
        _ => {
            return vec![];
        }
    };
    let capbility = Capabilities::from_string(&string);
    let item = match capbility {
        Capabilities::BROWSER => Browser::to_vector(),
        _ => vec![],
    };

    item.iter()
        .map(|item_| {
            CompletionItem::new_simple(item_.clone(), String::from("Browser").to_uppercase())
        })
        .collect()
}

fn complete_capbility_key() -> Vec<CompletionItem> {
    Capabilities::to_vector()
        .iter()
        .map(|capability| {
            CompletionItem::new_simple(capability.clone(), String::from("Capbilities"))
        })
        .collect()
}
