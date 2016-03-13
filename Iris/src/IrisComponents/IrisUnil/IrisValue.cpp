#include "IrisValue.h"
#include "IrisObject.h"

IrisValue::IrisValue()
{
}

IrisValue IrisValue::WrapObjectPointerToIrisValue(IrisObject* pObjectPointer) {
	IrisValue ivValue;
	ivValue.m_pObject = pObjectPointer;
	return ivValue;
};

//bool IrisValue::operator < (const IrisValue& ivValue) {
//	return m_pObject->GetObjectID() < ((IrisValue&)ivValue).GetObject()->GetObjectID();
//}

void* IrisValue::GetInstanceNativePointer() { return m_pObject->m_pNativeObject; }

IrisValue::~IrisValue()
{
}
