import sys
from argparse import ArgumentParser, Namespace
from pathlib import Path
from typing import List, Tuple


def parse_args(argv: List[str]) -> Namespace:
    parser = ArgumentParser()
    parser.add_argument("input_file", type=Path)
    args = parser.parse_args(argv)
    return args


def calculate_checksum(byte_count: int, address: int, data: bytes) -> int:
    """ Calculate checksum for record.
    """
    checksum = byte_count
    checksum += sum(b for b in address.to_bytes(1, "little"))
    checksum += sum(data)
    checksum = 0xFF - (checksum & 0xFF)
    return checksum


def parse_line(line: str) -> Tuple[int, int, int, bytes, int]:
    """ Parse SREC line

    Args:
        line (str) : Line to parse.

    Returns:
        Tuple of
            * record type
            * byte count
            * address
            * data
            * checksum
    """
    if line[0] != "S":
        raise SyntaxError(f"Invalid character '{line[0]}' at position 0 of line '{line}' (must be 'S')")

    for i, c in enumerate(line[1:]):
        if not ("0" <= c <= "9" or "A" <= c <= "F"):
            raise SyntaxError(f"Invalid character '{c}' at position {i}")

    if not "0" <= line[1] <= "9":
        raise SyntaxError(f"Expected record type [0, 9] at position 1 of line '{line}'")
    record_type = int(line[1])
    if record_type == 4:
        raise SyntaxError(f"Invalid record type 4 (reserved) of line '{line}'")

    byte_count = int(line[2:4], 16)
    expected_line_length = byte_count * 2 + 4
    if expected_line_length != len(line):
        raise SyntaxError(f"Expected line length {expected_line_length} (byte count 0x{byte_count:02X}) but "
                          f"line '{line}' has length {len(line)}")

    num_address_symbols_map = [4, 4, 6, 8, None, 4, 6, 8, 6, 4]
    num_address_symbols = num_address_symbols_map[record_type]
    index = 4
    end_index = index + num_address_symbols
    address = int(line[index : end_index], 16)
    index = end_index

    if record_type == 0 and address != 0:
        raise SyntaxError(f"Address must be 0 for S0 records (address 0x{address:X} in line '{line}')")

    num_data_bytes = byte_count * 2 - num_address_symbols - 2
    end_index = index + num_data_bytes
    data = bytes.fromhex(line[index : end_index])
    index = end_index

    end_index = index + 2
    checksum = int(line[index : end_index], 16)
    calculated_checksum = calculate_checksum(byte_count, address, data)
    if checksum != calculated_checksum:
        raise SyntaxError(f"Calculated checksum 0x{calculated_checksum:02X} for line '{line}' does not match "
                          f"checksum from line (0x{checksum:02X})")

    return record_type, byte_count, address, data, checksum


def validate_file_record_type(record_type: int, file_record_type: int) -> int:
    record_type_to_file_record_type = [None, 1, 2, 3, None, None, None, 3, 2, 1]
    expected_file_record_type = record_type_to_file_record_type[record_type]
    if not (file_record_type is None or expected_file_record_type is None):
        if file_record_type != expected_file_record_type:
            raise SyntaxError(f"SREC type already set to S{file_record_type}{10 - file_record_type} but "
                              f"this clashes with S{record_type}")
    return expected_file_record_type


def main(argv: List[str]):
    args = parse_args(argv)

    input_file: Path = args.input_file
    input_lines = input_file.read_text().strip().splitlines()

    header_records = []
    data_records = []
    start_address = None
    count = None

    file_record_type = None
    for line in input_lines:
        record_type, byte_count, address, data, checksum = parse_line(line)
        file_record_type = validate_file_record_type(record_type, file_record_type)
        match record_type:
            case 0:
                header_records.append(data)
            case 1 | 2 | 3:
                data_records.append((record_type, byte_count, address, data, checksum))
            case 5 | 6:
                if count is not None:
                    raise SyntaxError(f"Multiple count records found")
                count = address
            case 7 | 8 | 9:
                if start_address is not None:
                    raise SyntaxError(f"Multiple start addresses found")
                start_address = address

if __name__ == "__main__":
    main(sys.argv[1:])
