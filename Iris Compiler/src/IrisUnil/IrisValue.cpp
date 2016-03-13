#include "IrisUnil/IrisValue.h"
#include "IrisInterpreter/IrisStructure/IrisObject.h"

IrisValue::IrisValue()
{
}

IrisValue IrisValue::WrapObjectPointerToIrisValue(IIrisObject* pObjectPointer) {
	IrisValue ivValue;
	ivValue.m_pObject = pObjectPointer;
	return ivValue;
};

//bool IrisValue::operator < (const IrisValue& ivValue) {
//	return m_pObject->GetObjectID() < ((IrisValue&)ivValue).GetIrisObject()->GetObjectID();
//}

void* IrisValue::GetInstanceNativePointer() const {
	return m_pObject->GetNativeObject(); 
}

IrisValue::~IrisValue()
{
}
