#!/usr/bin/ruby

def read_bytecodes
    read_enum("BytecodeOpcode")
end

def read_types
    read_enum("BytecodeTypeKind")
end

def read_constpool_opcodes
    read_enum("ConstPoolOpcode")
end

def read_instruction_sets
    read_enum("InstructionSet")
end

def read_source_type_opcodes
    read_enum("SourceTypeOpcode")
end

def read_enum(name)
    values = []
    parse_line = false

    File.open('dora/src/bytecode/data.rs').each_line do |line|
        unless parse_line
            parse_line = true if line == "enumeration!(#{name} {\n" || line == "pub enum #{name} {\n"
            next
        end

        next if line.strip.empty?
        next if line.match(/^\s*\/\//)

        return values if line == "});\n" || line == "}\n"

        m = line.match(/^\s*([a-zA-Z0-9]+),?$/)

        unless m
            raise "illegal line: #{line.inspect}"
        end

        values.push(m[1])
    end

    raise "enum #{name} not found" unless parse_line

    values
end

def output
    bytecodes = read_bytecodes
    types = read_types
    constpool_opcodes = read_constpool_opcodes
    instruction_sets = read_instruction_sets
    source_type_opcodes = read_source_type_opcodes

    File.open('dora-boots/bytecode/opcode.dora', 'w') do |f|
        f.puts "// generated by tools/bytecode-gen.rb"
        f.puts

        opcode = 0

        for bytecode in bytecodes
            f.puts "@pub const BC_#{snake_case(bytecode)}: Int32 = #{opcode};"
            opcode += 1
        end

        f.puts
        type_code = 0

        for type in types
            f.puts "@pub const BC_TYPE_#{snake_case(type)}: Int32 = #{type_code};"
            type_code += 1
        end

        f.puts
        code = 0

        for opcode in constpool_opcodes
            f.puts "@pub const CONSTPOOL_OPCODE_#{snake_case(opcode)}: Int32 = #{code};"
            code += 1
        end

        f.puts
        f.puts "@pub fn bytecodeName(opcode: Int32): String {"

        for bytecode in bytecodes
            f.puts "  if opcode == BC_#{snake_case(bytecode)} { return #{bytecode.inspect}; }"
        end

        f.puts "  unreachable[String]()"
        f.puts "}"
        f.puts

        f.puts "@pub fn bytecodeTypeName(code: Int32): String {"

        for type in types
            f.puts "  if code == BC_TYPE_#{snake_case(type)} { return #{type.inspect}; }"
        end

        f.puts "  unreachable[String]()"
        f.puts "}"

        f.puts
        code = 0

        for opcode in instruction_sets
            f.puts "@pub const INSTRUCTION_SET_#{snake_case(opcode)}: Int32 = #{code};"
            code += 1
        end

        f.puts
        code = 0

        for opcode in source_type_opcodes
            f.puts "@pub const SOURCE_TYPE_OPCODE_#{snake_case(opcode)}: Int32 = #{code};"
            code += 1
        end
    end
end

def snake_case(name)
    result = name.gsub(/(.)([A-Z])/, '\1_\2')
    result.upcase
end

output
