use parser::token::Token;
use span::Span;
use tower_lsp::lsp_types::{SemanticToken, SemanticTokenType};

use crate::to_pos;

pub const SEMANTIC_TOKEN_TYPES: [SemanticTokenType; 11] = [
    SemanticTokenType::NUMBER,
    SemanticTokenType::VARIABLE,
    SemanticTokenType::KEYWORD,
    SemanticTokenType::STRING,
    SemanticTokenType::OPERATOR,
    SemanticTokenType::COMMENT,
    SemanticTokenType::FUNCTION,
    SemanticTokenType::PARAMETER,
    SemanticTokenType::TYPE,
    SemanticTokenType::PARAMETER,
    SemanticTokenType::PROPERTY,
];

fn match_token_type(token: &Token) -> SemanticTokenType {
    let token_type = &token.token_type;
    use parser::keywords::TokenType::*;
    use tower_lsp::lsp_types::SemanticTokenType as S;
    match token_type {
        NUMBER(_) => S::NUMBER,
        STRING(_) => S::STRING,
        IDENTIFIER(_) => S::VARIABLE,
        TESTCASE | TESTPLAN | TESTSUITE | TESTSTEPS | NONE | CAPABILITIES | PREREQUISITE => S::TYPE,
        NAVIGATE | CLICK | BACK | FORWARD | REFRESH | GET | WAIT | ASSERT | ENTER | CLOSE => {
            S::FUNCTION
        }
        ATTRIBUTE | ELEMENT | URL | TITLE | CURRENT => S::PARAMETER,
        IF | ELSE | TRUE | FALSE => S::KEYWORD,
        ASSIGN_OP
        | NEGATION
        | PLUS
        | MINUS
        | MULTIPLY
        | FORWARDSLASH
        | MODULUS
        | LEFT_PARAN
        | RIGHT_PARAN
        | L_CURLY_BRACE
        | R_CURLY_BRACE
        | EQUALITY
        | NOT_EQUAL
        | GREATER_THAN
        | LESSER_THAN
        | GREATER_THAN_EQUAL_TO
        | LESSER_THAN_EQUAL_TO
        | AND
        | OR => S::OPERATOR,
        FROM | TO | IN | NEW_LINE | EOF | ERROR => S::PROPERTY,
    }
}
pub fn update_semantic_tokens(tokens: &Vec<Token>) -> Vec<SemanticToken> {
    let mut semantic_tokens = vec![];
    let mut last_pos = (0, 0);
    for token in tokens {
        let stt = SEMANTIC_TOKEN_TYPES
            .iter()
            .position(|x| *x == match_token_type(&token))
            .map(|x| x as u32)
            .unwrap();
        let len = token.token_type.len() as u32;
        let pos = to_pos(token.span);

        if last_pos.0 == pos.0 {}
        let semantic_token = SemanticToken {
            delta_line: pos.0 - last_pos.0,
            delta_start: if pos.0 == last_pos.0 {
                pos.1 - last_pos.1
            } else {
                pos.0
            },
            length: len,
            token_type: stt,
            token_modifiers_bitset: 0,
        };
        semantic_tokens.push(semantic_token);
        last_pos = pos;
    }
    semantic_tokens
}
