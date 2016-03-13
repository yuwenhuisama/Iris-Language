#include "IrisUnil/IrisMemoryPool/IrisAbstractMemoryPool.h"

IrisAbstractMemoryPool::MemoryPoolList* IrisAbstractMemoryPool::s_lsPoolList;
IrisAbstractMemoryPool* IrisAbstractMemoryPool::sm_pInstance = nullptr;

IrisAbstractMemoryPool::MemoryPoolList * IrisAbstractMemoryPool::GetMemoryPoolList()
{
	return s_lsPoolList;
}

void IrisAbstractMemoryPool::SetMemoryPoolList(IrisAbstractMemoryPool::MemoryPoolList * pList)
{
	s_lsPoolList = pList;
}

void IrisAbstractMemoryPool::RegistMemoryPool(IrisAbstractMemoryPool * pMemoryPool)
{
	s_lsPoolList->push_back(pMemoryPool);
}

void IrisAbstractMemoryPool::Initialize()
{
	s_lsPoolList = new MemoryPoolList(RESERVATION_COUNT);
	for (size_t i = 0; i < RESERVATION_COUNT; ++i) {
		(*s_lsPoolList)[i] = nullptr;
	}
}

void IrisAbstractMemoryPool::ReleaseAllMemoryPool()
{
	for (auto pool : *s_lsPoolList) {
		if(pool) {
			pool->Release();
		}
	}
	delete s_lsPoolList;
}

IrisAbstractMemoryPool * IrisAbstractMemoryPool::GetCurrentMemoryPool()
{
	if (!sm_pInstance) {
		sm_pInstance = new IrisAbstractMemoryPool();
	}
	return sm_pInstance;
}

IrisAbstractMemoryPool * IrisAbstractMemoryPool::GetMemoryPool(size_t nIndex)
{
	return (*s_lsPoolList)[nIndex];
}

void IrisAbstractMemoryPool::SetMemoryPool(size_t nIndex, IrisAbstractMemoryPool * pPool)
{
	(*s_lsPoolList)[nIndex] = pPool;
}

void IrisAbstractMemoryPool::SetCurrentMemoryPool(IrisAbstractMemoryPool * pPool)
{
	sm_pInstance = pPool;
}

IrisAbstractMemoryPool::IrisAbstractMemoryPool()
{
}

void IrisAbstractMemoryPool::Release()
{
}

IrisAbstractMemoryPool::~IrisAbstractMemoryPool()
{
}
