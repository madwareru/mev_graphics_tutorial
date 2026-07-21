declare -a arr=(
  "ex00_red_picture"
  "ex01_draw_moon"
  "ex02_winding_number_triangle"
  "ex03_winding_trick_for_shapes"
  "ex04_barycentric_coordinates"
  "ex05_uv_coordinates"
  "ex06_texture_mapping_nearest"
  "ex07_texture_mapping_bilinear"
  "ex08_drawing_simple_unlit_3d_model"
  "ex09_visualize_normals"
  "ex0A_gouraud_shading"
  "ex11_mev_clear"
)
for i in "${arr[@]}"
do
  cargo build --release --example "$i"
  ./target/release/examples/"$i" > ./pictures/"$i".ppm
done