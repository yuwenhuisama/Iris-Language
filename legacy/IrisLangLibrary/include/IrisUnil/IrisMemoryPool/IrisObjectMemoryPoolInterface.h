#ifndef _H_IRISOBJECTMEMORYPOOLINTERFACE_
#define _H_IRISOBJECTMEMORYPOOLINTERFACE_
#include "IrisMemoryPool.h"
#include <thread>
#include <mutex>
using namespace std;

template <class T, int ID = -1>
class IrisObjectMemoryPoolInterface
{
protected:
	static recursive_mutex m_rmDataMutex;
	enum { nID = ID };

public:
	virtual ~IrisObjectMemoryPoolInterface();

	void* operator new(size_t size);
	void operator delete (void *p);
};

template <class T, int ID = -1>
recursive_mutex IrisObjectMemoryPoolInterface<T, ID>::m_rmDataMutex;

template <class T, int ID = -1>
IrisObjectMemoryPoolInterface<T, ID>::~IrisObjectMemoryPoolInterface()
{
}

template<class T, int ID = -1>
void * IrisObjectMemoryPoolInterface<T, ID>::operator new(size_t size)
 {
	lock_guard<recursive_mutex> lgLock(m_rmDataMutex);
	IrisMemoryPool<T>* pInstance = nullptr;
	if (nID != -1) {
		auto pMemory = IrisAbstractMemoryPool::GetCurrentMemoryPool();
		pInstance = static_cast<IrisMemoryPool<T>*>(pMemory->GetMemoryPool(nID));
		if (!pInstance) {
			IrisMemoryPool<T>::Generate();
			pInstance = IrisMemoryPool<T>::GetInstance();
			IrisAbstractMemoryPool::GetCurrentMemoryPool()->SetMemoryPool(nID, pInstance);
		}
	}
	else {
		pInstance = IrisMemoryPool<T>::GetInstance();
		if (!pInstance) {
			IrisMemoryPool<T>::Generate();
			IrisMemoryPool<T>::Regist();
			pInstance = IrisMemoryPool<T>::GetInstance();
		}
	}
	return pInstance->Malloc();
}

template<class T, int ID = -1>
void IrisObjectMemoryPoolInterface<T, ID>::operator delete(void * p)
{
	lock_guard<recursive_mutex> lgLock(m_rmDataMutex);
	IrisMemoryPool<T>* pInstance = nullptr;
	if (nID != -1) {
		auto pMemory = IrisAbstractMemoryPool::GetCurrentMemoryPool();
		pInstance = static_cast<IrisMemoryPool<T>*>(pMemory->GetMemoryPool(nID));
		if (!pInstance) {
			IrisMemoryPool<T>::Generate();
			pInstance = IrisMemoryPool<T>::GetInstance();
			IrisAbstractMemoryPool::GetCurrentMemoryPool()->SetMemoryPool(nID, pInstance);
		}
	}
	else {
		pInstance = IrisMemoryPool<T>::GetInstance();
		if (!pInstance) {
			IrisMemoryPool<T>::Generate();
			IrisMemoryPool<T>::Regist();
			pInstance = IrisMemoryPool<T>::GetInstance();
		}
	}
	return pInstance->Free(p);
}

#endif