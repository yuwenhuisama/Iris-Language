#ifndef _H_IRIMEMORYPOOL_
#define _H_IRIMEMORYPOOL_
#include "IrisAbstractMemoryPool.h"
#include <list>
#include <vector>
using namespace std;

template <class T>
class IrisSubMemoryPool;

template <class T>
class IrisMemoryPool :
	public IrisAbstractMemoryPool
{
private:
	static const size_t c_nBasePoolSlotCount = 32;
	//static const size_t c_nPoolSize = sizeof(T) * c_nPoolSlotCount;

private:
	list<char*> m_lsMallocedMemoryList;
	vector<char*> m_lsFreeSlotList;
	size_t m_nNextSlotCount = c_nBasePoolSlotCount;

private:
	static IrisMemoryPool<T>* s_pInstance;

public:
	static IrisMemoryPool<T>* Instance();

private:
	IrisMemoryPool();

public:
	virtual void* Malloc();
	virtual void Free(void* p);
	virtual void Release();
	virtual ~IrisMemoryPool();
};

#endif
