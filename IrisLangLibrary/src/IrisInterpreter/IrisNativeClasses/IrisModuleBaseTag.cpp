#include "IrisInterpreter/IrisNativeClasses/IrisModuleBaseTag.h"
#include "IrisInterpreter/IrisStructure/IrisModule.h"


IrisModuleBaseTag::IrisModuleBaseTag(IrisModule* pModule) : m_pModule(pModule)
{
}

void IrisModuleBaseTag::SetModule(IrisModule* pModule) {
	m_pModule = pModule;
}

const string& IrisModuleBaseTag::GetModuleName() {
#ifdef IR_USE_STL_STRING
	return m_pModule->GetModuleName();
#else
	return m_pModule->GetModuleName().GetSTLString();
#endif // IR_USE_STL_STRING
}

IrisModule* IrisModuleBaseTag::GetModule() {
	return m_pModule;
}

IrisModuleBaseTag::~IrisModuleBaseTag()
{
}
