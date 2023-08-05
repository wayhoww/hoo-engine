pub fn gltf_cube() -> &'static str {
    r#"
    {
        "asset" : {
            "generator" : "Khronos glTF Blender I/O v3.4.50",
            "version" : "2.0"
        },
        "scene" : 0,
        "scenes" : [
            {
                "name" : "Scene",
                "nodes" : [
                    0
                ]
            }
        ],
        "nodes" : [
            {
                "mesh" : 0,
                "name" : "Cube"
            }
        ],
        "materials" : [
            {
                "doubleSided" : true,
                "name" : "Material",
                "pbrMetallicRoughness" : {
                    "baseColorFactor" : [
                        0.800000011920929,
                        0.800000011920929,
                        0.800000011920929,
                        1
                    ],
                    "metallicFactor" : 0,
                    "roughnessFactor" : 0.5
                }
            }
        ],
        "meshes" : [
            {
                "name" : "Cube",
                "primitives" : [
                    {
                        "attributes" : {
                            "POSITION" : 0,
                            "TEXCOORD_0" : 1,
                            "NORMAL" : 2
                        },
                        "indices" : 3,
                        "material" : 0
                    }
                ]
            }
        ],
        "accessors" : [
            {
                "bufferView" : 0,
                "componentType" : 5126,
                "count" : 24,
                "max" : [
                    1,
                    1,
                    1
                ],
                "min" : [
                    -1,
                    -1,
                    -1
                ],
                "type" : "VEC3"
            },
            {
                "bufferView" : 1,
                "componentType" : 5126,
                "count" : 24,
                "type" : "VEC2"
            },
            {
                "bufferView" : 2,
                "componentType" : 5126,
                "count" : 24,
                "type" : "VEC3"
            },
            {
                "bufferView" : 3,
                "componentType" : 5123,
                "count" : 36,
                "type" : "SCALAR"
            }
        ],
        "bufferViews" : [
            {
                "buffer" : 0,
                "byteLength" : 288,
                "byteOffset" : 0,
                "target" : 34962
            },
            {
                "buffer" : 0,
                "byteLength" : 192,
                "byteOffset" : 288,
                "target" : 34962
            },
            {
                "buffer" : 0,
                "byteLength" : 288,
                "byteOffset" : 480,
                "target" : 34962
            },
            {
                "buffer" : 0,
                "byteLength" : 72,
                "byteOffset" : 768,
                "target" : 34963
            }
        ],
        "buffers" : [
            {
                "byteLength" : 840,
                "uri" : "data:application/octet-stream;base64,AACAPwAAgD8AAIC/AACAPwAAgD8AAIC/AACAPwAAgD8AAIC/AACAPwAAgL8AAIC/AACAPwAAgL8AAIC/AACAPwAAgL8AAIC/AACAPwAAgD8AAIA/AACAPwAAgD8AAIA/AACAPwAAgD8AAIA/AACAPwAAgL8AAIA/AACAPwAAgL8AAIA/AACAPwAAgL8AAIA/AACAvwAAgD8AAIC/AACAvwAAgD8AAIC/AACAvwAAgD8AAIC/AACAvwAAgL8AAIC/AACAvwAAgL8AAIC/AACAvwAAgL8AAIC/AACAvwAAgD8AAIA/AACAvwAAgD8AAIA/AACAvwAAgD8AAIA/AACAvwAAgL8AAIA/AACAvwAAgL8AAIA/AACAvwAAgL8AAIA/AAAgPwAAAD8AACA/AAAAPwAAID8AAAA/AADAPgAAAD8AAMA+AAAAPwAAwD4AAAA/AAAgPwAAgD4AACA/AACAPgAAID8AAIA+AADAPgAAgD4AAMA+AACAPgAAwD4AAIA+AAAgPwAAQD8AACA/AABAPwAAYD8AAAA/AAAAPgAAAD8AAMA+AABAPwAAwD4AAEA/AAAgPwAAAAAAACA/AACAPwAAYD8AAIA+AAAAPgAAgD4AAMA+AAAAAAAAwD4AAIA/AAAAAAAAAAAAAIC/AAAAAAAAgD8AAACAAACAPwAAAAAAAACAAAAAAAAAgL8AAACAAAAAAAAAAAAAAIC/AACAPwAAAAAAAACAAAAAAAAAAAAAAIA/AAAAAAAAgD8AAACAAACAPwAAAAAAAACAAAAAAAAAgL8AAACAAAAAAAAAAAAAAIA/AACAPwAAAAAAAACAAACAvwAAAAAAAACAAAAAAAAAAAAAAIC/AAAAAAAAgD8AAACAAAAAAAAAgL8AAACAAACAvwAAAAAAAACAAAAAAAAAAAAAAIC/AAAAAAAAAAAAAIA/AACAvwAAAAAAAACAAAAAAAAAgD8AAACAAAAAAAAAgL8AAACAAAAAAAAAAAAAAIA/AACAvwAAAAAAAACAAQAOABQAAQAUAAcACgAGABIACgASABYAFwATAAwAFwAMABAADwADAAkADwAJABUABQACAAgABQAIAAsAEQANAAAAEQAAAAQA"
            }
        ]
    }
    "#
}
