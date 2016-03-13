#include "IrisMemoryPool.h"

#ifndef _CPP_IRISMEMORYPOOL_
#define _CPP_IRISMEMORYPOOL_

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
inline IrisMemoryPool<T>* IrisMemoryPool<T>::Instance()
{
	if (!s_pInstance) {
		s_pInstance = new IrisMemoryPool<T>();
	}
	return s_pInstance;
}

template<class T>
inline IrisMemoryPool<T>::IrisMemoryPool() : m_lsFreeSlotList(c_nBasePoolSlotCount)
{
	IrisAbstractMemoryPool::RegistMemoryPool(this);
}

template<class T>
inline IrisMemoryPool<T>::~IrisMemoryPool()
{
}

#endif