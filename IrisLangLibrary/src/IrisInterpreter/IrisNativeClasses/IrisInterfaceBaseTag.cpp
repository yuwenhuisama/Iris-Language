#include "IrisInterpreter/IrisNativeClasses/IrisInterfaceBaseTag.h"
#include "IrisInterpreter/IrisStructure/IrisInterface.h"


IrisInterfaceBaseTag::IrisInterfaceBaseTag(IrisInterface* pInterface) : m_pInterface(pInterface)
{
}

const string& IrisInterfaceBaseTag::GetInterfaceName() {
#ifdef IR_USE_STL_STRING
	return m_pInterface->GetInterfaceName();
#else
	return m_pInterface->GetInterfaceName().GetSTLString();
#endif // USE_STL_STRING
}

void IrisInterfaceBaseTag::SetInterface(IrisInterface* pInterface) {
	m_pInterface = pInterface;
}

IrisInterface* IrisInterfaceBaseTag::GetInterface() {
	return m_pInterface;
}

IrisInterfaceBaseTag::~IrisInterfaceBaseTag()
{
}
