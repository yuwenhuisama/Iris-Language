#include "IrisInterpreter/IrisNativeClasses/IrisMethodBaseTag.h"
#include "IrisInterpreter/IrisStructure/IrisMethod.h"

IrisMethodBaseTag::IrisMethodBaseTag(IrisMethod* pMethod) : m_pMethod(pMethod)
{
}

const string& IrisMethodBaseTag::GetMethodName() {
#ifdef IR_USE_STL_STRING
	return m_pMethod->GetMethodName();
#else
	return m_pMethod->GetMethodName().GetSTLString();
#endif // USE_STL_STRING
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
