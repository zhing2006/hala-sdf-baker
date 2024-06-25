cbuffer GlobalUniformBuffer : register(b0, space0) {
  float4x4 v_mtx;   // The view matrix
  float4x4 p_mtx;   // The projection matrix
  float4x4 vp_mtx;  // The view-projection matrix
};

cbuffer ObjectUniformBuffer : register(b0, space1) {
  float4x4 m_mtx;     // The model matrix
  float4x4 i_m_mtx;   // The inverse model matrix
  float4x4 mv_mtx;    // The model-view matrix
  float4x4 t_mv_mtx;  // The transposed model-view matrix
  float4x4 it_mv_mtx; // The inverse transposed model-view matrix
  float4x4 mvp_mtx;   // The model-view-projection matrix
};