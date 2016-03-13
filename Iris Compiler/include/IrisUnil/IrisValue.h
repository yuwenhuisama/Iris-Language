#ifndef _H_IRISVALUE_
#define _H_IRISVALUE_
#include <vector>
using namespace std;

class IIrisObject;
//class IrisObject;
class IrisValue
{
private:
	IIrisObject* m_pObject;
public:
	IrisValue();
	~IrisValue();

	inline IIrisObject* GetIrisObject() { return m_pObject; }
	inline void SetIrisObject(IIrisObject* pObject) { m_pObject = pObject; }

public:
	static IrisValue WrapObjectPointerToIrisValue(IIrisObject* pObjectPointer);
	void* GetInstanceNativePointer() const;

	bool operator == (const IrisValue& ivValue){
		return ivValue.m_pObject == m_pObject;
	}

	bool operator != (const IrisValue& ivValue) {
		return !(*this == ivValue);
	}
};

//typedef vector<IrisValue> IrisValues;

#endif