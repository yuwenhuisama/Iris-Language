#include "IrisInterpreter/IrisNativeClasses/IrisClassBaseTag.h"
#include "IrisInterpreter/IrisStructure/IrisClass.h"

IrisClassBaseTag::IrisClassBaseTag(IrisClass* pClass) : m_pClass(pClass)
{
}

const string& IrisClassBaseTag::GetThisClassName() {
#ifdef IR_USE_STL_STRING
	return m_pClass->GetClassName();
#else
	return m_pClass->GetClassName().GetSTLString();
#endif // USE_STL_STRING
}

void IrisClassBaseTag::SetClass(IrisClass* pClass) {
	m_pClass = pClass;
}

IrisClass* IrisClassBaseTag::GetClass() {
	return m_pClass;
}

IrisClassBaseTag::~IrisClassBaseTag()
{
}
