#include "IrisUnil/IrisHeap.h"
#include "IrisInterpreter/IrisStructure/IrisObject.h"

IrisHeap::IrisHeap()
{
}

void IrisHeap::AddObject(IrisObject* pObject) {
	m_stHeap.insert(pObject);
}

void IrisHeap::RemoveObject(IrisObject* pObject) {
	m_stHeap.erase(pObject);
}

unordered_set<IrisObject*>& IrisHeap::GetHeapSet() {
	return m_stHeap;
}

IrisHeap::~IrisHeap()
{
}
