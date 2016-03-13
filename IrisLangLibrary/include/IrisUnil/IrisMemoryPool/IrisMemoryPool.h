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
	static IrisMemoryPool<T>* GetInstance();
	static void Generate();
	static void Regist();

private:
	IrisMemoryPool();

public:
	virtual void* Malloc();
	virtual void Free(void* p);
	virtual void Release();
	virtual ~IrisMemoryPool();
};

template <class T>
IrisMemoryPool<T>* IrisMemoryPool<T>::s_pInstance = nullptr;

template<class T>
void * IrisMemoryPool<T>::Malloc()
{
	char* pData = nullptr;
	if (m_lsFreeSlotList.empty()) {
		pData = new char[m_nNextSlotCount * sizeof(T)];
		m_lsMallocedMemoryList.push_back(pData);
		m_lsFreeSlotList.reserve(m_nNextSlotCount);
		for (size_t i = 0; i < m_nNextSlotCount; ++i) {
			m_lsFreeSlotList.push_back(pData + i * sizeof(T));
		}
		// The same strategy as stl::vector
		m_nNextSlotCount += m_nNextSlotCount;
	}
	pData = m_lsFreeSlotList.back();
	m_lsFreeSlotList.pop_back();
	return static_cast<void*>(pData);
}

template<class T>
void IrisMemoryPool<T>::Free(void * p)
{
	m_lsFreeSlotList.push_back(static_cast<char*>(p));
}

template<class T>
inline void IrisMemoryPool<T>::Release()
{
	for (auto pointer : m_lsMallocedMemoryList) {
		delete pointer;
	}
}

template<class T>
inline IrisMemoryPool<T>* IrisMemoryPool<T>::GetInstance()
{
	return s_pInstance;
}

template<class T>
inline void IrisMemoryPool<T>::Generate()
{
	s_pInstance = new IrisMemoryPool<T>();
}

template<class T>
inline void IrisMemoryPool<T>::Regist()
{
	IrisAbstractMemoryPool::GetCurrentMemoryPool()->RegistMemoryPool(s_pInstance);
}

template<class T>
inline IrisMemoryPool<T>::IrisMemoryPool()
{
	m_lsFreeSlotList.reserve(c_nBasePoolSlotCount);
}

template<class T>
inline IrisMemoryPool<T>::~IrisMemoryPool()
{
}

#endif
