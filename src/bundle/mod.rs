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

pub fn default_shader() -> &'static str {
    r#"
    // #define UNIFORM_BIND_GROUP_Material 0
    // #define UNIFORM_BIND_GROUP_DrawCall 1
    // #define UNIFORM_BIND_GROUP_Pass 2
    // #define UNIFORM_BIND_GROUP_Task 3
    // #define UNIFORM_BIND_GROUP_Global 4
    // 有无 const 语法？
    
    const xx = 0;
    
    struct VertexOut {
        @builtin(position) position : vec4f,
    
        @location(0) uv0 : vec2f,
        @location(1) normal_local : vec3f
    };
    
    struct FragmentOut {
        @location(0) color : vec4f,
        // @builtin(frag_depth) depth: f32,
    };
    
    struct DrawCallUniform {
        transform_m: mat4x4f,
        transform_mv: mat4x4f,
        transform_mvp: mat4x4f,
    
        // color1: f32,
        // color2: f32,
    };
    
    @group(0) @binding(1) var<uniform> cDrawCall: DrawCallUniform;
    
    
    @vertex
    fn vsMain_base(
        @location(0) pos: vec3f, 
        @location(1) normal: vec3f, 
        @location(2) uv: vec2f
    ) -> VertexOut {
        // cDrawCall.matrix_projection * 
        var vertex_out: VertexOut;
        vertex_out.position = cDrawCall.transform_mvp * vec4f(pos.xyz, 1.0);
        vertex_out.uv0 = uv;
        vertex_out.normal_local = normal;
        
        return vertex_out;
    }
    
    @fragment
    fn fsMain_base(vertex_out: VertexOut) -> FragmentOut {
        // w: linear depth
        // w/z: 0~1, 0 means near
    
        var fragment_out: FragmentOut;
        fragment_out.color = vec4f(abs(vertex_out.normal_local.xyz) + 0.3, 1.0);
        // fragment_out.depth = vertex_out.position.w / vertex_out.position.z;
        return fragment_out;
    }
    "#
}
