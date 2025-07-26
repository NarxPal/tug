use crate::instruction::Instruction;

pub fn parse_line(line: &str) -> Option<Instruction> {
    let trimmed = line.trim();
    if trimmed.starts_with("FROM ") {
        Some(Instruction::From(trimmed[5..].to_string()))
    } else if trimmed.starts_with("RUN ") {
        Some(Instruction::Run(trimmed[4..].to_string()))
    } else if trimmed.starts_with("COPY ") {
        let parts: Vec<&str> = trimmed[5..].split_whitespace().collect();
        if parts.len() == 2 {
            Some(Instruction::Copy {
                src: parts[0].to_string(),
                dest: parts[1].to_string(),
            })
        } else {
            None
        }
    } else if trimmed.starts_with("CMD ") {
        Some(Instruction::Cmd(trimmed[4..].to_string()))
    } else if trimmed.starts_with("WORKDIR ") {
        Some(Instruction::Workdir(trimmed[8..].to_string()))
    } else if trimmed.starts_with("EXPOSE ") {
        trimmed[7..].parse::<u16>().ok().map(Instruction::Expose)
    } else if trimmed.starts_with("ENV ") {
        let parts: Vec<&str> = trimmed[4..].splitn(2, '=').collect();
        if parts.len() == 2 {
            Some(Instruction::Env {
                key: parts[0].trim().to_string(),
                value: parts[1].trim().to_string(),
            })
        } else {
            None
        }
    } else if trimmed.starts_with("ENTRYPOINT ") {
        Some(Instruction::EntryPoint(trimmed[11..].to_string()))
    } else if trimmed.starts_with("ADD ") {
        let parts: Vec<&str> = trimmed[4..].split_whitespace().collect();
        if parts.len() == 2 {
            Some(Instruction::Add {
                src: parts[0].to_string(),
                dest: parts[1].to_string(),
            })
        } else {
            None
        }
    } else {
        None
    }
}

pub fn parse_file(input: &str) -> Vec<Instruction> {
    input.lines().filter_map(|line| parse_line(line)).collect()
}
