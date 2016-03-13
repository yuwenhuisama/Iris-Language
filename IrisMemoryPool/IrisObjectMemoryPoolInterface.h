#ifndef _H_IRISOBJECTMEMORYPOOLINTERFACE_
#define _H_IRISOBJECTMEMORYPOOLINTERFACE_
#include "IrisMemoryPool.h"
#include "IrisMemoryPool.cpp"

template <class T>
class IrisObjectMemoryPoolInterface
{
protected:
	IrisSubMemoryPool<T>* m_pSubMemory = nullptr;

public:
	virtual ~IrisObjectMemoryPoolInterface();

	void* operator new(size_t size);
	void operator delete (void *p);
};

template <class T>
IrisObjectMemoryPoolInterface<T>::~IrisObjectMemoryPoolInterface()
{
}

template<class T>
void * IrisObjectMemoryPoolInterface<T>::operator new(size_t size)
{
	return IrisMemoryPool<T>::Instance()->Malloc();
}

template<class T>
void IrisObjectMemoryPoolInterface<T>::operator delete(void * p)
{
	IrisMemoryPool<T>::Instance()->Free(p);
}

#endif