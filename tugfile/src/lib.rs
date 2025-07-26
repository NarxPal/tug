pub mod instruction;
pub mod parser;

#[cfg(test)]
mod tests {
    use super::instruction::Instruction;
    use super::parser::parse_file;

    #[test]
    fn test_parse_full_tugfile() {
        let input = r#"
    FROM alpine
    WORKDIR /app
    COPY . /app
    RUN apk add --no-cache curl
    EXPOSE 8080
    CMD ["./start.sh"]
"#;

        let result: Vec<Instruction> = parse_file(input);
        assert_eq!(result.len(), 6);
    }
}
