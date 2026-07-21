use std::collections::HashMap;

/// Obj file loaded from a string as is.
struct ObjFile {
    /// Vertex coordinates
    vs: Vec<(f32, f32, f32)>,
    /// Texture coordinates
    vts: Vec<(f32, f32)>,
    /// Normals
    vns: Vec<(f32, f32, f32)>,
    /// Indices for vertices. Be careful, they are 1-based
    v_indices: Vec<[u16; 3]>,
    /// Indices for texture coordinates. Be careful, they are 1-based
    vt_indices: Vec<[u16; 3]>,
    /// Indices for texture normals. Be careful, they are 1-based
    vn_indices: Vec<[u16; 3]>,
}
impl ObjFile {
    /// Loads an obj file from a source string. Only the one specific subset is supported,
    /// where the model should have only vertices, texture coordinates, and normals. Indices
    /// are expected to be triplets in a form `v_id/vt_id/vn_id`.
    fn load_from_string(source: &str) -> Result<Self, &'static str> {
        let mut vs = Vec::new();
        let mut vts = Vec::new();
        let mut vns = Vec::new();
        let mut v_indices = Vec::new();
        let mut vt_indices = Vec::new();
        let mut vn_indices = Vec::new();
        let mut lines = source.lines();
        while let Some(line) = lines.next() {
            let mut parts = line.split_whitespace();
            let Some(tag) = parts.next() else { continue; };
            match tag {
                "v" => {
                    let a = parts.next()
                        .ok_or("Broken vertex line spotted, expected 3 coords but got none")?
                        .parse()
                        .map_err(|_| "Expected vertex component to be a floating point number")?;
                    let b = parts.next()
                        .ok_or("Broken vertex line spotted, expected 3 coords but got only one")?
                        .parse()
                        .map_err(|_| "Expected vertex component to be a floating point number")?;
                    let c = parts.next()
                        .ok_or("Broken vertex line spotted, expected 3 coords but got only two")?
                        .parse()
                        .map_err(|_| "Expected vertex component to be a floating point number")?;

                    vs.push((a, b, c));
                }
                "vt" => {
                    let u = parts.next()
                        .ok_or("Broken texture coord line spotted, expected 2 coords but got none")?
                        .parse()
                        .map_err(|_| "Expected texture coordinate to be a floating point number")?;
                    let v = parts.next()
                        .ok_or("Broken texture coord line spotted, expected 2 coords but got only one")?
                        .parse()
                        .map_err(|_| "Expected texture coordinate to be a floating point number")?;

                    vts.push((u, v));
                }
                "vn" => {
                    let a = parts.next()
                        .ok_or("Broken normal spotted, expected 3 coords but got none")?
                        .parse()
                        .map_err(|_| "Expected normal component to be a floating point number")?;
                    let b = parts.next()
                        .ok_or("Broken normal spotted, expected 3 coords but got only one")?
                        .parse()
                        .map_err(|_| "Expected normal component to be a floating point number")?;
                    let c = parts.next()
                        .ok_or("Broken normal spotted, expected 3 coords but got only two")?
                        .parse()
                        .map_err(|_| "Expected normal component to be a floating point number")?;

                    vns.push((a, b, c));
                }
                "f" => {
                    let triplet_0 = parts.next()
                        .ok_or("Broken face line spotted, expected 3 or 4 index triplets but got none")?;
                    let triplet_1 = parts.next()
                        .ok_or("Broken face line spotted, expected 3 or 4 index triplets but got only one")?;
                    let triplet_2 = parts.next()
                        .ok_or("Broken face line spotted, expected 3 or 4 index triplets but got only two")?;
                    let (v_index_0, vt_index_0, vn_index_0) = parse_index_triplet(triplet_0)?;
                    let (v_index_1, vt_index_1, vn_index_1) = parse_index_triplet(triplet_1)?;
                    let (v_index_2, vt_index_2, vn_index_2) = parse_index_triplet(triplet_2)?;
                    v_indices.push([v_index_0, v_index_1, v_index_2]);
                    vt_indices.push([vt_index_0, vt_index_1, vt_index_2]);
                    vn_indices.push([vn_index_0, vn_index_1, vn_index_2]);

                    if let Some(triplet_3) = parts.next() {
                        let (v_index_3, vt_index_3, vn_index_3) = parse_index_triplet(triplet_3)?;
                        v_indices.push([v_index_0, v_index_2, v_index_3]);
                        vt_indices.push([vt_index_0, vt_index_2, vt_index_3]);
                        vn_indices.push([vn_index_0, vn_index_2, vn_index_3]);
                    }
                    fn parse_index_triplet(triplet_str: &str) -> Result<(u16, u16, u16), &'static str> {
                        let mut parts = triplet_str.split('/');
                        let v_index = parts.next()
                            .ok_or("Broken index triplet spotted, expected 3 indices but got none")?
                            .parse()
                            .map_err(|_| "Expected vertex index to be a number")?;
                        let vt_index = parts.next()
                            .ok_or("Broken index triplet spotted, expected 2 indices but got only one")?
                            .parse()
                            .map_err(|_| "Expected uv index to be a number")?;
                        let vn_index = parts.next()
                            .ok_or("Broken index triplet spotted, expected 3 indices but got only two")?
                            .parse()
                            .map_err(|_| "Expected normal index to be a number")?;
                        Ok((v_index, vt_index, vn_index))
                    }
                }
                _ => {}
            }
        }
        Ok(Self { vs, vts, vns, v_indices, vt_indices, vn_indices })
    }
}

/// Obj model subset with vertices, texture coordinates, and normals.
#[derive(Clone)]
pub struct ObjModel {
    /// Vertex coordinates
    vs: Vec<glam::Vec4>,
    /// Texture coordinates
    vts: Vec<glam::Vec2>,
    /// Normals
    vns: Vec<glam::Vec4>,
    /// Indices. 0-based.
    indices: Vec<[u16; 3]>,
}
impl ObjModel {
    /// Loads an obj file from a source string. Only the one specific subset is supported,
    /// where the model should have only vertices, texture coordinates, and normals. Indices
    /// are expected to be triplets in a form `v_id/vt_id/vn_id`.
    pub fn load_from_string(source: &str) -> Result<Self, &'static str> {
        let source = ObjFile::load_from_string(source)?;
        let mut vs = Vec::new();
        let mut vts = Vec::new();
        let mut vns = Vec::new();
        let mut indices = Vec::new();

        let mut mapper = HashMap::new();

        let source_vs = source.vs;
        let source_vts = source.vts;
        let source_vns = source.vns;

        for v_id in source.v_indices.iter().flatten().copied() {
            if source_vs.len() < v_id as usize {
                return Err("Vertex index out of bounds");
            }
        }

        for vt_id in source.vt_indices.iter().flatten().copied() {
            if source_vts.len() < vt_id as usize {
                return Err("Texture coord index out of bounds");
            }
        }

        for vn_id in source.vn_indices.iter().flatten().copied() {
            if source_vns.len() < vn_id as usize {
                return Err("Normal index out of bounds");
            }
        }

        let source_v_ids = source.v_indices
            .iter()
            .copied()
            .map(|it| it.map(|component| component - 1 ));
        let source_vt_ids = source.vt_indices
            .iter()
            .copied()
            .map(|it| it.map(|component| component - 1 ));
        let source_vn_ids = source.vn_indices
            .iter()
            .copied()
            .map(|it| it.map(|component| component - 1 ));

        for ((v_triangle, vt_triangle), vn_triangle) in source_v_ids.zip(source_vt_ids).zip(source_vn_ids) {
            let new_triangle: [u16; 3] = std::array::from_fn(|i| {
                let lookup_id = (v_triangle[i], vt_triangle[i], vn_triangle[i]);
                match mapper.get(&lookup_id).copied() {
                    Some(index) => index,
                    None => {
                        let v = source_vs[v_triangle[i] as usize];
                        let vt = source_vts[vt_triangle[i] as usize];
                        let vn = source_vns[vn_triangle[i] as usize];
                        vs.push(glam::vec4(v.0, v.1, v.2, 1.0));
                        vts.push(glam::vec2(vt.0, vt.1));
                        vns.push(glam::vec4(vn.0, vn.1, vn.2, 0.0).normalize_or_zero());
                        let idx = mapper.len() as u16;
                        mapper.insert(lookup_id, idx);
                        idx
                    }
                }
            });
            indices.push(new_triangle);
        }

        Ok(Self { vs, vts, vns, indices })
    }
    /// Vertex coordinates
    pub fn vs(&self) -> &[glam::Vec4] { &self.vs }
    /// Texture coordinates
    pub fn vts(&self) -> &[glam::Vec2] { &self.vts }
    /// Normals
    pub fn vns(&self) -> &[glam::Vec4] { &self.vns }
    /// Indices, 0-based.
    pub fn indices(&self) -> &[[u16; 3]] { &self.indices }
}