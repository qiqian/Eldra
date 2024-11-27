#pragma once
#include <include/gpu/ganesh/d3d/GrD3DTypes.h>
#include "D3D12MemAlloc.h" // this file is externals


// Custom class D3DAMDMemoryAllocator for assigning shared texture handles
class D3DAMDMemoryAllocator : public GrD3DMemoryAllocator {
public:
    static sk_sp<D3DAMDMemoryAllocator> Make(IDXGIAdapter* adapter, ID3D12Device* device);

    ~D3DAMDMemoryAllocator() override { fAllocator->Release(); }
    // this function will called
    gr_cp<ID3D12Resource> createResource(D3D12_HEAP_TYPE, const D3D12_RESOURCE_DESC*,
        D3D12_RESOURCE_STATES initialResourceState,
        sk_sp<GrD3DAlloc>* allocation,
        const D3D12_CLEAR_VALUE*) override;

    gr_cp<ID3D12Resource> createAliasingResource(sk_sp<GrD3DAlloc>& allocation,
        uint64_t localOffset,
        const D3D12_RESOURCE_DESC*,
        D3D12_RESOURCE_STATES initialResourceState,
        const D3D12_CLEAR_VALUE*) override;

    class Alloc : public GrD3DAlloc {
    public:
        Alloc(D3D12MA::Allocation* allocation) : fAllocation(allocation) {}
        ~Alloc() override {
            fAllocation->Release();
        }
    private:
        friend class D3DAMDMemoryAllocator;
        D3D12MA::Allocation* fAllocation;
    };

private:
    D3DAMDMemoryAllocator(D3D12MA::Allocator* allocator) : fAllocator(allocator) {}

    D3D12MA::Allocator* fAllocator;
};
