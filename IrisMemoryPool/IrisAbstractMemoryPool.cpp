#include "IrisAbstractMemoryPool.h"

IrisAbstractMemoryPool::MemoryPoolList IrisAbstractMemoryPool::s_lsPoolList;

void IrisAbstractMemoryPool::RegistMemoryPool(IrisAbstractMemoryPool * pMemoryPool)
{
	s_lsPoolList.push_back(pMemoryPool);
}

void IrisAbstractMemoryPool::ReleaseAllMemoryPool()
{
	for (auto pool : s_lsPoolList) {
		pool->Release();
	}
}

IrisAbstractMemoryPool::IrisAbstractMemoryPool()
{
}


IrisAbstractMemoryPool::~IrisAbstractMemoryPool()
{
}
