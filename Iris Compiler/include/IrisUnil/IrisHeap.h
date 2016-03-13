#ifndef _H_IRISHEAP_
#define _H_IRISHEAP_

#include <unordered_set>
using namespace std;

class IrisObject;
class IrisHeap
{
private:
	unordered_set<IrisObject*> m_stHeap;

public:
	void AddObject(IrisObject* pObject);
	void RemoveObject(IrisObject* pObject);
	unordered_set<IrisObject*>& GetHeapSet();

	IrisHeap();
	~IrisHeap();
};

#endif