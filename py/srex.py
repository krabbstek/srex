from __future__ import annotations

import sys
from argparse import ArgumentParser, Namespace
from pathlib import Path
from typing import List, Tuple


class SRecordFile:
    def __init__(self):
        self.file_path = None
        self.header_data = bytes()
        self.data = bytes()
        self.address_indices: List[Tuple[int, int]] = []

    @classmethod
    def from_str(cls, srecord_str: str) -> SRecordFile:
        srecord_file = cls()
        for line in srecord_str.splitlines():
            record_type, byte_count, address, data, checksum = parse_line(line)
            if record_type == 0:
                srecord_file.header_data += data
            elif record_type in (1, 2, 3):
                srecord_file.data += data
                for i, (start_address, end_address) in enumerate(srecord_file.address_indices):
                    if start_address <= address < end_address:
                        raise Exception(f"Duplicate data: 0x{address:0X}")
                    if address == end_address:
                        srecord_file.address_indices[i] = (start_address, end_address + len(data))
                        break
                else:
                    srecord_file.address_indices.append((address, address + len(data)))
        return srecord_file


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
    srecord_file = SRecordFile.from_str(input_file.read_text())

if __name__ == "__main__":
    main(sys.argv[1:])
