#ifndef _H_IRISABSTRACTMEMORYPOOL_
#define _H_IRISABSTRACTMEMORYPOOL_
#include "IrisMemoryPoolDefines.h"
#include <vector>
using namespace std;
class IrisAbstractMemoryPool
{
private:
	typedef vector<IrisAbstractMemoryPool*> MemoryPoolList;
	
private:
	static MemoryPoolList* s_lsPoolList;

private:
	static IrisAbstractMemoryPool* sm_pInstance;

public:
	void RegistMemoryPool(IrisAbstractMemoryPool* pMemoryPool);
	void ReleaseAllMemoryPool();

	IrisAbstractMemoryPool* GetMemoryPool(size_t nIndex);
	void SetMemoryPool(size_t nIndex, IrisAbstractMemoryPool* pPool);

public:
	static void Initialize();
	static IrisAbstractMemoryPool* GetCurrentMemoryPool();
	static void SetCurrentMemoryPool(IrisAbstractMemoryPool* pPool);
	static MemoryPoolList* GetMemoryPoolList();
	static void SetMemoryPoolList(MemoryPoolList* pList);

protected:
	IrisAbstractMemoryPool();

public:
	virtual void Release();
	virtual ~IrisAbstractMemoryPool();
};

#endif