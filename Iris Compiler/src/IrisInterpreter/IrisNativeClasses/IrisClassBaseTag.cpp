#include "IrisInterpreter/IrisNativeClasses/IrisClassBaseTag.h"
#include "IrisInterpreter/IrisStructure/IrisClass.h"

IrisClassBaseTag::IrisClassBaseTag(IrisClass* pClass) : m_pClass(pClass)
{
}

const string& IrisClassBaseTag::GetThisClassName() {
	return m_pClass->GetClassName();
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
