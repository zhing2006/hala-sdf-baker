// A single iteration of Bob Jenkins' One-At-A-Time hashing algorithm.
uint jenkins_hash(uint x) {
  x += (x << 10u);
  x ^= (x >>  6u);
  x += (x <<  3u);
  x ^= (x >> 11u);
  x += (x << 15u);
  return x;
}

// Compound versions of the hashing algorithm.
uint jenkins_hash(uint2 v) {
  return jenkins_hash(v.x ^ jenkins_hash(v.y));
}

uint jenkins_hash(uint3 v) {
  return jenkins_hash(v.x ^ jenkins_hash(v.yz));
}

uint jenkins_hash(uint4 v) {
  return jenkins_hash(v.x ^ jenkins_hash(v.yzw));
}

// Construct a float with half-open range [0, 1) using low 23 bits.
// All zeros yields 0, all ones yields the next smallest representable value below 1.
float construct_float(int m) {
  const int ieee_mantissa = 0x007FFFFF; // Binary FP32 mantissa bitmask
  const int iieee_one    = 0x3F800000;  // 1.0 in FP32 IEEE

  m &= ieee_mantissa; // Keep only mantissa bits (fractional part)
  m |= iieee_one;     // Add fractional part to 1.0

  float  f = asfloat(m);  // Range [1, 2)
  return f - 1;           // Range [0, 1)
}

float construct_float(uint m) {
  return construct_float(asint(m));
}

// Pseudo-random value in half-open range [0, 1). The distribution is reasonably uniform.
// Ref: https://stackoverflow.com/a/17479300
float generate_hashed_random_float(uint x) {
  return construct_float(jenkins_hash(x));
}

float generate_hashed_random_float(uint2 v) {
  return construct_float(jenkins_hash(v));
}

float generate_hashed_random_float(uint3 v) {
  return construct_float(jenkins_hash(v));
}

float generate_hashed_random_float(uint4 v) {
  return construct_float(jenkins_hash(v));
}