#include "IrisInterpreter/IrisNativeClasses/IrisMethodBaseTag.h"
#include "IrisInterpreter/IrisStructure/IrisMethod.h"

IrisMethodBaseTag::IrisMethodBaseTag(IrisMethod* pMethod) : m_pMethod(pMethod)
{
}

const string& IrisMethodBaseTag::GetMethodName() {
	return m_pMethod->GetMethodName();
}

void IrisMethodBaseTag::SetMethod(IrisMethod* pMethod) {
	m_pMethod = pMethod;
}
IrisMethod * IrisMethodBaseTag::GetMethod()
{
	return m_pMethod;
}
::

IrisMethodBaseTag::~IrisMethodBaseTag()
{
}
