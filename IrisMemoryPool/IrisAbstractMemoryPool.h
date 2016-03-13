#ifndef _H_IRISABSTRACTMEMORYPOOL_
#define _H_IRISABSTRACTMEMORYPOOL_
#include <list>
using namespace std;
class IrisAbstractMemoryPool
{
private:
	typedef list<IrisAbstractMemoryPool*> MemoryPoolList;
	
private:
	static MemoryPoolList s_lsPoolList;

public:
	static void RegistMemoryPool(IrisAbstractMemoryPool* pMemoryPool);
	static void ReleaseAllMemoryPool();

public:
	IrisAbstractMemoryPool();
	virtual void Release() = 0;
	virtual ~IrisAbstractMemoryPool() = 0;
};

#endif