struct Camera {
  float3 position;  // camera position
  float3 right;     // camera right vector
  float3 up;        // camera up vector
  float3 forward;   // camera forward vector
  float yfov;       // vertical field of view
  float focal_distance_or_xmag; // focal distance for perspective camera and xmag for orthographic camera
  float aperture_or_ymag; // aperture size for perspective camera and ymag for orthographic camera
  uint type;      // 0 - perspective camera, 1 - orthographic camera
};

struct Light {
  float3 intensity;
  // For point light, position is the position.
  // For directional light, position is unused.
  // For spot light, quad light and sphere light, position is the position.
  float3 position;
  // For point light, u is unused.
  // For directional light and spot light, u is the direction.
  // For quad light, u is the right direction and length.
  // For sphere light, u is unused.
  float3 u;
  // For point light, v is unused.
  // For directional light, v.x is the cosine of the cone angle.
  // For spot light, v.x is the cosine of the inner cone angle, v.y is the cosine of the outer cone angle.
  // For quad light, v is the up direction and length.
  // For sphere light, v is unused.
  float3 v;
  // For point light, directional light, spot light and quad light, radius is unused.
  // For sphere light, radius is the radius.
  float radius;
  // For point light, directional light and spot light, area is unused.
  // For quad light and sphere light, area is the area.
  float area;
  // light type: 0 - point, 1 - directional, 2 - spot, 3 - quad, 4 - sphere
  int type;
};

struct Medium {
  float3 color;
  float density;
  float anisotropy;
  uint type;
  float padding0;
  float padding1;
};

struct Material {
  Medium medium;

  [[vk::offset(32)]]
  float3 base_color;
  float opacity;

  float3 emission;
  float anisotropic;

  float metallic;
  float roughness;
  float subsurface;
  float specular_tint;

  float sheen;
  float sheen_tint;
  float clearcoat;
  float clearcoat_roughness;

  float3 clearcoat_tint;
  float specular_transmission;

  float ior;
  float ax;
  float ay;
  uint base_color_map_index;

  uint normal_map_index;
  uint metallic_roughness_map_index;
  uint emission_map_index;
  uint type;
};
