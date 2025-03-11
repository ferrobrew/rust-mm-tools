meta:
  id: mm_adf
  file-extension: adf
  endian: le
seq:
  - id: header
    type: header
instances:
  hashes:
    if: header.hash_count > 0
    pos: header.hash_offset
    repeat: expr
    repeat-expr: header.hash_count
    type: u4
  string_lengths:
    if: header.string_count > 0
    pos: header.string_offset
    repeat: expr
    repeat-expr: header.string_count
    type: u1
  strings:
    if: header.string_count > 0
    pos: header.string_offset + header.string_count
    repeat: expr
    repeat-expr: header.string_count
    type: strz
    encoding: ASCII
    size: string_lengths[_index] + 1
  instances:
    if: header.instance_count > 0
    pos: header.instance_offset
    repeat: expr
    repeat-expr: header.instance_count
    type: instance_definition
  types:
    if: header.type_count > 0
    pos: header.type_offset
    repeat: expr
    repeat-expr: header.type_count
    type: type_definition
enums:
  type:
    0: scalar
    1: struct
    2: pointer
    3: array
    4: inline_array
    5: string
    6: recursive
    7: bitfield
    8: enum
    9: string_hash
    10: deferred
  type_flags:
    0: none
    1: pod_read
    2: pod_write
    32768: finalize
  member_flags:
    0: none
    1: default_value
    2: default_instance
  scalar_type:
    0: signed
    1: unsigned
    2: float
types:
  header:
    seq:
      - id: magic
        contents: ' FDA'
      - id: version
        type: u4
      - id: instance_count
        type: u4
      - id: instance_offset
        type: u4
      - id: type_count
        type: u4
      - id: type_offset
        type: u4
      - id: hash_count
        type: u4
      - id: hash_offset
        type: u4
      - id: string_count
        type: u4
      - id: string_offset
        type: u4
      - id: file_size
        type: u4
      - id: padding
        size: 20
      - id: description
        type: strz
        encoding: ASCII
  type_definition:
    seq:
      - id: type
        type: u4
        enum: type
      - id: size
        type: u4
      - id: alignment
        type: u4
      - id: type_hash
        type: u4
      - id: name
        type: u8
      - id: flags
        type: u2
        enum: type_flags
      - id: scalar_type
        type: u2
        enum: scalar_type
      - id: element_type_hash
        type: u4
      - id: element_length
        type: u4
      - id: member_count
        type: u4
      - id: members
        type: member_definition
        if: type == type::struct
        repeat: expr
        repeat-expr: member_count
      - id: enums
        type: enum_definition
        if: type == type::enum
        repeat: expr
        repeat-expr: member_count
    instances:
      name_string:
        value: _parent.strings[name]
  member_definition:
    seq:
      - id: name
        type: u8
      - id: type_hash
        type: u4
      - id: alignment
        type: u4
      - id: offsets
        type: u4
      - id: flags
        type: u4
        enum: member_flags
      - id: value
        type: u8
    instances:
      name_string:
        value: _parent._parent.strings[name]
      offset:
        value: (offsets & 0x00FFFFFF) >> 0
      bit_offset:
        value: (offsets & 0xFF000000) >> 24
  enum_definition:
    seq:
      - id: name
        type: u8
      - id: value
        type: s4
    instances:
      name_string:
        value: _parent._parent.strings[name]
  instance_definition:
    seq:
      - id: name_hash
        type: u4
      - id: type_hash
        type: u4
      - id: payload_offset
        type: u4
      - id: payload_size
        type: u4
      - id: name
        type: u8
    instances:
      name_string:
        value: _parent.strings[name]
      payload:
        pos: payload_offset
        type: u1
        repeat: expr
        repeat-expr: payload_size