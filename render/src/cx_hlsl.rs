use crate::cx::*;
use crate::cx_dx11::*;
use std::ffi;
use winapi::shared::{dxgiformat};
use winapi::um::{d3d11, d3dcommon};
use wio::com::ComPtr;

#[derive(Default, Clone)]
pub struct AssembledHlslShader {
    pub geometry_slots: usize,
    pub instance_slots: usize,
    pub geometries: Vec<ShVar>,
    pub instances: Vec<ShVar>,
    pub uniforms_dr: Vec<ShVar>,
    pub uniforms_dl: Vec<ShVar>,
    pub uniforms_cx: Vec<ShVar>,
    pub texture_slots: Vec<ShVar>,
    pub rect_instance_props: RectInstanceProps,
    pub named_uniform_props: NamedProps,
    pub named_instance_props: NamedProps,
    pub hlsl: String,
}

#[derive(Clone)]
pub struct CompiledShader {
    pub shader_id: usize,
    pub geom_vbuf: D3d11Buffer,
    pub geom_ibuf: D3d11Buffer,
    pub geometry_slots: usize,
    pub instance_slots: usize,
    pub rect_instance_props: RectInstanceProps,
    pub named_uniform_props: NamedProps,
    pub named_instance_props: NamedProps,
    pub pixel_shader: ComPtr<d3d11::ID3D11PixelShader>,
    pub vertex_shader: ComPtr<d3d11::ID3D11VertexShader>,
    pub pixel_shader_blob: ComPtr<d3dcommon::ID3DBlob>,
    pub vertex_shader_blob: ComPtr<d3dcommon::ID3DBlob>,
    pub input_layout: ComPtr<d3d11::ID3D11InputLayout>
}

impl Cx {
    pub fn hlsl_compile_all_shaders(&mut self, d3d11: &D3d11) {
        for sh in &self.shaders {
            let shc = Self::hlsl_compile_shader(&sh, d3d11);
            if let Ok(shc) = shc {
                self.compiled_shaders.push(CompiledShader {
                    shader_id: self.compiled_shaders.len(),
                    ..shc
                });
            }
            else if let Err(err) = shc {
                panic!("Got shader compile error: {}", err.msg);
            }
        };
    }
    
    pub fn hlsl_type(ty: &str) -> String {
        match ty.as_ref() {
            "float" => "float".to_string(),
            "vec2" => "float2".to_string(),
            "vec3" => "float3".to_string(),
            "vec4" => "float4".to_string(),
            "mat2" => "float2x2".to_string(),
            "mat3" => "float3x3".to_string(),
            "mat4" => "float4x4".to_string(),
            "texture2d" => "Texture2D".to_string(),
            ty => ty.to_string()
        }
    }
    
    pub fn hlsl_assemble_struct(lead: &str, name: &str, vars: &Vec<ShVar>, semantic: &str, field: &str, post: &str) -> String {
        let mut out = String::new();
        out.push_str(lead);
        out.push_str(" ");
        out.push_str(name);
        out.push_str(post);
        out.push_str("{\n");
        out.push_str(field);
        for (index, var) in vars.iter().enumerate() {
            out.push_str("  ");
            out.push_str(&Self::hlsl_type(&var.ty));
            out.push_str(" ");
            out.push_str(&var.name);
            if semantic.len()>0 {
                out.push_str(": ");
                out.push_str(&format!("{}{}", semantic, var.name.to_uppercase()));
                //out.push_str(&format!("{}", index));
            }
            out.push_str(";\n")
        };
        out.push_str("};\n\n");
        out
    }
    pub fn hlsl_init_struct(vars: &Vec<ShVar>, field: &str) -> String {
        let mut out = String::new();
        out.push_str("{\n");
        out.push_str(field);
        for (index, var) in vars.iter().enumerate() {
            out.push_str(match Self::hlsl_type(&var.ty).as_ref() {
                "float" => "0.0",
                "float2" => "float2(0.0,0.0)",
                "float3" => "float3(0.0,0.0,0.0)",
                "float4" => "float4(0.0,0.0,0.0,0.0)",
                _ => "",
            });
            out.push_str(",")
        };
        out.push_str("}");
        out
    }
    
    pub fn hlsl_assemble_texture_slots(textures: &Vec<ShVar>) -> String {
        let mut out = String::new();
        for (i, tex) in textures.iter().enumerate() {
            out.push_str("Texture2D ");
            out.push_str(&tex.name);
            out.push_str(&format!(": register(t{});\n", i));
        };
        out
    }
    
    pub fn hlsl_assemble_shader(sh: &Shader) -> Result<AssembledHlslShader, SlErr> {
        
        let mut hlsl_out = String::new();
        
        hlsl_out.push_str("SamplerState DefaultTextureSampler{Filter = MIN_MAG_MIP_LINEAR;AddressU = Wrap;AddressV=Wrap;};\n");
        
        // ok now define samplers from our sh.
        let texture_slots = sh.flat_vars(ShVarStore::Texture);
        let geometries = sh.flat_vars(ShVarStore::Geometry);
        let instances = sh.flat_vars(ShVarStore::Instance);
        let mut varyings = sh.flat_vars(ShVarStore::Varying);
        let locals = sh.flat_vars(ShVarStore::Local);
        let uniforms_cx = sh.flat_vars(ShVarStore::UniformCx);
        let uniforms_dl = sh.flat_vars(ShVarStore::UniformDl);
        let uniforms_dr = sh.flat_vars(ShVarStore::Uniform);
        
        // lets count the slots
        let geometry_slots = sh.compute_slot_total(&geometries);
        let instance_slots = sh.compute_slot_total(&instances);
        //let varying_slots = sh.compute_slot_total(&varyings);
        hlsl_out.push_str(&Self::hlsl_assemble_texture_slots(&texture_slots));
        
        hlsl_out.push_str(&Self::hlsl_assemble_struct("struct", "_Geom", &geometries, "GEOM_", "", ""));
        hlsl_out.push_str(&Self::hlsl_assemble_struct("struct", "_Inst", &instances, "INST_", "", ""));
        hlsl_out.push_str(&Self::hlsl_assemble_struct("cbuffer", "_Uni_Cx", &uniforms_cx, "", "", ": register(b0)"));
        hlsl_out.push_str(&Self::hlsl_assemble_struct("cbuffer", "_Uni_Dl", &uniforms_dl, "", "", ": register(b1)"));
        hlsl_out.push_str(&Self::hlsl_assemble_struct("cbuffer", "_Uni_Dr", &uniforms_dr, "", "", ": register(b2)"));
        hlsl_out.push_str(&Self::hlsl_assemble_struct("struct", "_Loc", &locals, "", "", ""));
        
        // we need to figure out which texture slots exist
        // we need to figure out which texture slots exist
        // mtl_out.push_str(&Self::assemble_constants(&texture_slots));
        
        let mut const_cx = SlCx {
            depth: 0,
            target: SlTarget::Constant,
            defargs_fn: "".to_string(),
            defargs_call: "".to_string(),
            call_prefix: "_".to_string(),
            shader: sh,
            scope: Vec::new(),
            fn_deps: Vec::new(),
            fn_done: Vec::new(),
            auto_vary: Vec::new()
        };
        let consts = sh.flat_consts();
        for cnst in &consts {
            let const_init = assemble_const_init(cnst, &mut const_cx) ?;
            hlsl_out.push_str("#define ");
            hlsl_out.push_str(" ");
            hlsl_out.push_str(&cnst.name);
            hlsl_out.push_str(" (");
            hlsl_out.push_str(&const_init.sl);
            hlsl_out.push_str(")\n");
        }
        
        let mut vtx_cx = SlCx {
            depth: 0,
            target: SlTarget::Vertex,
            defargs_fn: "inout _Loc _loc, inout _Vary _vary, in _Geom _geom, in _Inst _inst".to_string(),
            defargs_call: "_loc, _vary, _geom, _inst".to_string(),
            call_prefix: "_".to_string(),
            shader: sh,
            scope: Vec::new(),
            fn_deps: vec!["vertex".to_string()],
            fn_done: Vec::new(),
            auto_vary: Vec::new()
        };
        
        let vtx_fns = assemble_fn_and_deps(sh, &mut vtx_cx) ?;
        let mut pix_cx = SlCx {
            depth: 0,
            target: SlTarget::Pixel,
            defargs_fn: "inout _Loc _loc, inout _Vary _vary".to_string(),
            defargs_call: "_loc, _vary".to_string(),
            call_prefix: "_".to_string(),
            shader: sh,
            scope: Vec::new(),
            fn_deps: vec!["pixel".to_string()],
            fn_done: vtx_cx.fn_done,
            auto_vary: Vec::new()
        };
        
        let pix_fns = assemble_fn_and_deps(sh, &mut pix_cx) ?;
        
        // lets add the auto_vary ones to the varyings struct
        for auto in &pix_cx.auto_vary {
            varyings.push(auto.clone());
        }
        hlsl_out.push_str(&Self::hlsl_assemble_struct("struct", "_Vary", &varyings, "VARY_", "  float4 hlsl_position : SV_POSITION;\n", ""));
        
        hlsl_out.push_str("//Vertex shader\n");
        hlsl_out.push_str(&vtx_fns);
        hlsl_out.push_str("//Pixel shader\n");
        hlsl_out.push_str(&pix_fns);
        
        // lets define the vertex shader
        hlsl_out.push_str("_Vary _vertex_shader(_Geom _geom, _Inst _inst, uint inst_id: SV_InstanceID){\n");
        hlsl_out.push_str("  _Loc _loc = ");
        hlsl_out.push_str(&Self::hlsl_init_struct(&locals, ""));
        hlsl_out.push_str(";\n");
        hlsl_out.push_str("  _Vary _vary = ");
        hlsl_out.push_str(&Self::hlsl_init_struct(&varyings, "float4(0.0,0.0,0.0,0.0),"));
        hlsl_out.push_str(";\n");
        hlsl_out.push_str("  _vary.hlsl_position = _vertex(");
        hlsl_out.push_str(&vtx_cx.defargs_call);
        hlsl_out.push_str(");\n\n");
        
        for auto in pix_cx.auto_vary {
            if let ShVarStore::Geometry = auto.store {
                hlsl_out.push_str("       _vary.");
                hlsl_out.push_str(&auto.name);
                hlsl_out.push_str(" = _geom.");
                hlsl_out.push_str(&auto.name);
                hlsl_out.push_str(";\n");
            }
            else if let ShVarStore::Instance = auto.store {
                hlsl_out.push_str("       _vary.");
                hlsl_out.push_str(&auto.name);
                hlsl_out.push_str(" = _inst.");
                hlsl_out.push_str(&auto.name);
                hlsl_out.push_str(";\n");
            }
        }
        
        hlsl_out.push_str("       return _vary;\n");
        hlsl_out.push_str("};\n");
        // then the fragment shader
        hlsl_out.push_str("float4 _pixel_shader(_Vary _vary) : SV_TARGET{\n");
        hlsl_out.push_str("  _Loc _loc = ");
        hlsl_out.push_str(&Self::hlsl_init_struct(&locals, ""));
        hlsl_out.push_str(";\n");
        hlsl_out.push_str("  return _pixel(");
        hlsl_out.push_str(&pix_cx.defargs_call);
        hlsl_out.push_str(");\n};\n");
        
        if sh.log != 0 {
            println!("---- HLSL shader -----");
            let lines = hlsl_out.split('\n');
            for (index, line) in lines.enumerate() {
                println!("{} {}", index + 1, line);
            }
        }
        
        Ok(AssembledHlslShader {
            rect_instance_props: RectInstanceProps::construct(sh, &instances),
            named_instance_props: NamedProps::construct(sh, &instances, false),
            named_uniform_props: NamedProps::construct(sh, &uniforms_dr, true),
            geometries: geometries,
            instances: instances,
            geometry_slots: geometry_slots,
            instance_slots: instance_slots,
            uniforms_dr: uniforms_dr,
            uniforms_dl: uniforms_dl,
            uniforms_cx: uniforms_cx,
            texture_slots: texture_slots,
            hlsl: hlsl_out
        })
    }
    
    fn slots_to_dxgi_format(slots: usize) -> u32 {
        match slots {
            1 => dxgiformat::DXGI_FORMAT_R32_FLOAT,
            2 => dxgiformat::DXGI_FORMAT_R32G32_FLOAT,
            3 => dxgiformat::DXGI_FORMAT_R32G32B32_FLOAT,
            4 => dxgiformat::DXGI_FORMAT_R32G32B32A32_FLOAT,
            _ => panic!("slots_to_dxgi_format unsupported slotcount {}", slots)
        }
    }
    
    pub fn hlsl_compile_shader(sh: &Shader, d3d11: &D3d11) -> Result<CompiledShader, SlErr> {
        let ash = Self::hlsl_assemble_shader(sh) ?;
        
        let vs_blob = d3d11.compile_shader("vs", "_vertex_shader".as_bytes(), ash.hlsl.as_bytes()) ?;
        let ps_blob = d3d11.compile_shader("ps", "_pixel_shader".as_bytes(), ash.hlsl.as_bytes()) ?;
        
        let vs = d3d11.create_vertex_shader(&vs_blob) ?;
        let ps = d3d11.create_pixel_shader(&ps_blob) ?;
        
        let mut layout_desc = Vec::new();
        let geom_named = NamedProps::construct(sh, &ash.geometries, false);
        let inst_named = NamedProps::construct(sh, &ash.instances, false);
        let mut strings = Vec::new();
        
        for (index, geom) in geom_named.props.iter().enumerate() {
            strings.push(ffi::CString::new(format!("GEOM_{}", geom.name.to_uppercase())).unwrap());
            layout_desc.push(d3d11::D3D11_INPUT_ELEMENT_DESC {
                SemanticName: strings.last().unwrap().as_ptr() as *const _,
                SemanticIndex: 0,
                Format: Self::slots_to_dxgi_format(geom.slots),
                InputSlot: 0,
                AlignedByteOffset: (geom.offset * 4) as u32,
                InputSlotClass: d3d11::D3D11_INPUT_PER_VERTEX_DATA,
                InstanceDataStepRate: 0
            })
        }
        
        for (index, inst) in inst_named.props.iter().enumerate() {
            strings.push(ffi::CString::new(format!("INST_{}", inst.name.to_uppercase())).unwrap());
            layout_desc.push(d3d11::D3D11_INPUT_ELEMENT_DESC {
                SemanticName: strings.last().unwrap().as_ptr() as *const _,
                SemanticIndex: 0,
                Format: Self::slots_to_dxgi_format(inst.slots),
                InputSlot: 1,
                AlignedByteOffset: (inst.offset * 4) as u32,
                InputSlotClass: d3d11::D3D11_INPUT_PER_INSTANCE_DATA,
                InstanceDataStepRate: 1
            })
        }
        
        let input_layout = d3d11.create_input_layout(&vs_blob, &layout_desc) ?;
        
        Ok(CompiledShader {
            shader_id: 0,
            vertex_shader: vs,
            pixel_shader: ps,
            vertex_shader_blob: vs_blob,
            pixel_shader_blob: ps_blob,
            input_layout: input_layout,
            geometry_slots: ash.geometry_slots,
            instance_slots: ash.instance_slots,
            named_instance_props: ash.named_instance_props.clone(),
            named_uniform_props: ash.named_uniform_props.clone(),
            rect_instance_props: ash.rect_instance_props.clone(),
            geom_ibuf: {
                let mut geom_ibuf = D3d11Buffer {..Default::default()};
                geom_ibuf.update_with_u32_index_data(d3d11, &sh.geometry_indices);
                geom_ibuf
            },
            geom_vbuf: {
                let mut geom_vbuf = D3d11Buffer {..Default::default()};
                geom_vbuf.update_with_f32_vertex_data(d3d11, &sh.geometry_vertices);
                geom_vbuf
            }
        })
    }
}


impl<'a> SlCx<'a> {
    pub fn map_call(&self, name: &str, args: &Vec<Sl>) -> MapCallResult {
        match name {
            "sample2d" => { // transform call to
                let base = &args[0];
                let coord = &args[1];
                return MapCallResult::Rewrite(
                    format!("{}.Sample(DefaultTextureSampler,{})", base.sl, coord.sl),
                    "vec4".to_string()
                )
            },
            "color" => {
                let col = color(&args[0].sl);
                return MapCallResult::Rewrite(
                    format!("float4({},{},{},{})", col.r, col.g, col.b, col.a),
                    "vec4".to_string()
                );
            },
            "mix" => {
                return MapCallResult::Rename("lerp".to_string())
            },
            "dfdx" => {
                return MapCallResult::Rename("ddx".to_string())
            },
            "dfdy" => {
                return MapCallResult::Rename("ddy".to_string())
            },
            _ => return MapCallResult::None
        }
    }
    
    pub fn mat_mul(&self, left: &str, right: &str) -> String {
        format!("mul({},{})", left, right)
    }
    
    pub fn map_type(&self, ty: &str) -> String {
        Cx::hlsl_type(ty)
    }
    
    pub fn map_var(&mut self, var: &ShVar) -> String {
        //let mty = Cx::hlsl_type(&var.ty);
        match var.store {
            ShVarStore::Uniform => return var.name.clone(), //format!("_uni_dr.{}", var.name),
            ShVarStore::UniformDl => return var.name.clone(), //format!("_uni_dl.{}", var.name),
            ShVarStore::UniformCx => return var.name.clone(), //format!("_uni_cx.{}", var.name),
            ShVarStore::Instance => {
                if let SlTarget::Pixel = self.target {
                    if self.auto_vary.iter().find( | v | v.name == var.name).is_none() {
                        self.auto_vary.push(var.clone());
                    }
                    return format!("_vary.{}", var.name);
                }
                else {
                    return format!("_inst.{}", var.name);
                }
            },
            ShVarStore::Geometry => {
                if let SlTarget::Pixel = self.target {
                    if self.auto_vary.iter().find( | v | v.name == var.name).is_none() {
                        self.auto_vary.push(var.clone());
                    }
                    return format!("_vary.{}", var.name);
                }
                else {
                    
                    return format!("_geom.{}", var.name);
                }
            },
            ShVarStore::Texture => return var.name.clone(),
            ShVarStore::Local => return format!("_loc.{}", var.name),
            ShVarStore::Varying => return format!("_vary.{}", var.name),
        }
    }
}