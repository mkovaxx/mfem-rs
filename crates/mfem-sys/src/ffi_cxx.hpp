#include "mfem.hpp"
#include "cxx.h"

#ifndef FFI_CXX_H
#define FFI_CXX_H

using namespace mfem;

template<typename T>
Operator const& upcast_to_operator(T const& x) {
    return x;
}

template<typename T>
Operator& upcast_to_operator_mut(T& x) {
    return x;
}

Operator const& OperatorHandle_operator(OperatorHandle const& x) {
    return *x;
}

Operator& OperatorHandle_operator_mut(OperatorHandle& x) {
    return *x;
}

SparseMatrix const& OperatorHandle_ref_SparseMatrix(OperatorHandle const& x) {
    return *x.As<SparseMatrix>();
}

std::unique_ptr<OperatorHandle>
SparseMatrix_to_OperatorHandle(SparseMatrix *x) {
    return std::make_unique<OperatorHandle>(x, false);
}

using mfem_Array_int_AutocxxConcrete = Array<int>;

template<typename T>
std::unique_ptr<Array<T>> array_with_len(int size) {
    return std::make_unique<Array<T>>(size);
}

template<typename T>
std::unique_ptr<Array<T>> array_copy(Array<T> const& src) {
    return std::make_unique<Array<T>>(src);
}

template<typename T>
std::unique_ptr<Array<T>> array_from_slice(T* data, int len, bool own_data) {
    return std::make_unique<Array<T>>(data, len, own_data);
}

using Element_Type = Element::Type;

using c_void = void;

std::unique_ptr<FunctionCoefficient>
new_FunctionCoefficient(rust::Fn<double(mfem::Vector const &, void*)> f,
                        void *d)
{
  std::function<real_t(const Vector &)> F =
      [d = std::move(d), f = std::move(f)](mfem::Vector const &x) {
      return f(x, d);
  };
  return std::make_unique<FunctionCoefficient>(F);
}


#endif // FFI_CXX_H
